//! Quarantine round-trips: quarantine -> list -> restore / delete.

use griffon_scanner::scanner_quarantine::Quarantine;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Fresh, empty directory under the system temp dir, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "griffon_q_test_{}_{}_{}",
            tag,
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Returns (workspace dir, quarantine) with the quarantine stored in `<ws>/q`.
fn setup(tag: &str) -> (TempDir, Quarantine) {
    let ws = TempDir::new(tag);
    let q = Quarantine::new(&ws.path().join("q")).unwrap();
    (ws, q)
}

fn write(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

#[test]
fn new_creates_directory() {
    let ws = TempDir::new("new");
    let dir = ws.path().join("nested/quarantine");
    let q = Quarantine::new(&dir).unwrap();
    assert!(dir.is_dir());
    assert!(q.list_sorted().is_empty());
}

#[test]
fn quarantine_moves_file_and_writes_manifest() {
    let (ws, q) = setup("move");
    let file = ws.path().join("evil.bin");
    write(&file, b"malicious payload");
    let original = file.canonicalize().unwrap();

    let dest = q.quarantine_file(&file).unwrap();

    assert!(!file.exists(), "original file should be gone");
    assert!(dest.exists(), "quarantined copy should exist");
    assert_eq!(std::fs::read(&dest).unwrap(), b"malicious payload");
    assert_eq!(dest.extension().unwrap(), "quarantined");

    let manifest = dest.with_extension("json");
    assert!(manifest.exists(), "manifest should be written next to it");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(manifest).unwrap()).unwrap();
    let expected_hash = hex::encode(Sha256::digest(b"malicious payload"));
    assert_eq!(json["original_hash"], expected_hash.as_str());
    assert_eq!(Path::new(json["original_path"].as_str().unwrap()), original);
    let name = dest.file_name().unwrap().to_str().unwrap();
    assert!(
        name.ends_with(&format!("_{}.quarantined", &expected_hash[..16])),
        "file name embeds hash prefix: {name}"
    );
}

#[test]
fn quarantine_missing_file_fails() {
    let (ws, q) = setup("missing");
    let err = q.quarantine_file(&ws.path().join("nope.bin")).unwrap_err();
    assert!(err.contains("does not exist"), "{err}");
}

#[test]
fn quarantine_virtual_archive_path_quarantines_the_archive() {
    let (ws, q) = setup("virtual");
    let archive = ws.path().join("bundle.zip");
    write(&archive, b"PK fake zip");

    let virtual_path = PathBuf::from(format!("{}!/inner/eicar.com", archive.display()));
    q.quarantine_file(&virtual_path).unwrap();

    assert!(!archive.exists(), "the containing archive is quarantined");
    let items = q.list_sorted();
    assert_eq!(items.len(), 1);
    assert!(items[0].original_path.ends_with("bundle.zip"));
}

#[test]
fn list_ignores_non_manifest_and_corrupt_files() {
    let (_ws, q) = setup("list");
    write(&q.dir.join("garbage.txt"), b"hello");
    write(&q.dir.join("broken.json"), b"{ not json");
    assert!(q.list_sorted().is_empty());
}

#[test]
fn quarantine_then_restore_roundtrip() {
    let (ws, q) = setup("restore");
    let file = ws.path().join("sub/dir/sample.txt");
    write(&file, b"restore me");
    let original = file.canonicalize().unwrap();

    q.quarantine_file(&file).unwrap();
    // Remove the parent dir: restore must recreate it.
    std::fs::remove_dir_all(ws.path().join("sub")).unwrap();

    let items = q.list_sorted();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].original_path, original);

    let restored = q.restore_file(&items[0].quarantine_name).unwrap();
    assert_eq!(restored, original);
    assert_eq!(std::fs::read(&original).unwrap(), b"restore me");
    assert!(q.list_sorted().is_empty(), "manifest removed after restore");
    assert_eq!(std::fs::read_dir(&q.dir).unwrap().count(), 0);
}

#[test]
fn restore_unknown_name_fails() {
    let (_ws, q) = setup("restore_unknown");
    let err = q.restore_file("does_not_exist.quarantined").unwrap_err();
    assert!(err.contains("Failed to read manifest"), "{err}");
}

#[test]
fn delete_quarantined_removes_both_files() {
    let (ws, q) = setup("delete");
    let file = ws.path().join("gone.bin");
    write(&file, b"delete me");

    q.quarantine_file(&file).unwrap();
    let name = q.list_sorted().remove(0).quarantine_name;

    q.delete_quarantined_file(&name).unwrap();
    assert!(q.list_sorted().is_empty());
    assert_eq!(std::fs::read_dir(&q.dir).unwrap().count(), 0);
    assert!(!file.exists(), "delete must not restore the file");
}

#[test]
fn batch_operations_reject_empty_input() {
    let (_ws, q) = setup("empty");
    assert!(q.quarantine_files(&[]).is_err());
    assert!(q.restore_files(&[]).is_err());
    assert!(q.delete_quarantined_files(&[]).is_err());
}

#[test]
fn batch_quarantine_succeeds_partially() {
    let (ws, q) = setup("partial");
    let good = ws.path().join("good.bin");
    write(&good, b"one");
    let bad = ws.path().join("missing.bin");

    let done = q.quarantine_files(&[good.clone(), bad]).unwrap();
    assert_eq!(done.len(), 1);
    assert!(!good.exists());
}

#[test]
fn batch_quarantine_fails_when_all_fail() {
    let (ws, q) = setup("allfail");
    let err = q
        .quarantine_files(&[ws.path().join("a"), ws.path().join("b")])
        .unwrap_err();
    assert!(err.contains("Failed to quarantine all 2"), "{err}");
}

#[test]
fn list_is_sorted_newest_first() {
    let (_ws, q) = setup("sorted");
    for (name, ts) in [
        (
            "20250101_000000_aaaaaaaaaaaaaaaa",
            "2025-01-01T00:00:00+00:00",
        ),
        (
            "20260101_000000_bbbbbbbbbbbbbbbb",
            "2026-01-01T00:00:00+00:00",
        ),
        (
            "20240101_000000_cccccccccccccccc",
            "2024-01-01T00:00:00+00:00",
        ),
    ] {
        let manifest = serde_json::json!({
            "original_path": format!("/tmp/{name}"),
            "quarantined_at": ts,
            "original_hash": "00",
        });
        write(
            &q.dir.join(format!("{name}.json")),
            manifest.to_string().as_bytes(),
        );
    }

    let items = q.list_sorted();
    let dates: Vec<_> = items.iter().map(|i| &i.quarantined_at[..4]).collect();
    assert_eq!(dates, ["2026", "2025", "2024"]);
    assert_eq!(
        items[0].quarantine_name,
        "20260101_000000_bbbbbbbbbbbbbbbb.quarantined"
    );
}
