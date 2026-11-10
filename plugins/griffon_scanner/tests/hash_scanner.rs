//! Hash signature database and SHA-256 hash matching.

use griffon_scanner::scanner_engine::data_type::Severity;
use griffon_scanner::scanner_engine::hash_scanner::{SignatureDb, hash};
use std::path::{Path, PathBuf};

/// sha256("abc"), from FIPS 180-2.
const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn temp_file(name: &str, content: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!("griffon_hash_{}_{}", std::process::id(), name));
    std::fs::write(&path, content).unwrap();
    path
}

fn db_with(name: &str, content: &str) -> SignatureDb {
    let path = temp_file(name, content.as_bytes());
    let db = SignatureDb::load(&path).unwrap();
    std::fs::remove_file(path).ok();
    db
}

#[test]
fn hash_file_matches_known_vector() {
    let path = temp_file("abc", b"abc");
    assert_eq!(hash::hash_file(&path).unwrap(), ABC_SHA256);
    std::fs::remove_file(path).ok();
}

#[test]
fn hash_file_missing_is_error() {
    assert!(hash::hash_file(Path::new("/definitely/not/here")).is_err());
}

#[test]
fn load_skips_comments_and_blank_lines() {
    let db = db_with(
        "db_comments",
        "# header comment\n\n   \n  aaaa  \n# another\nbbbb\n",
    );
    assert_eq!(db.count(), 2);
    assert!(db.contains("aaaa"), "surrounding whitespace is trimmed");
    assert!(db.contains("bbbb"));
    assert!(!db.contains("# header comment"));
}

#[test]
fn lookup_is_case_insensitive() {
    let db = db_with("db_case", &ABC_SHA256.to_uppercase());
    assert!(db.contains(ABC_SHA256));
    assert!(db.contains(&ABC_SHA256.to_uppercase()));
}

#[test]
fn duplicates_are_counted_once() {
    let db = db_with("db_dup", "abcd\nABCD\nabcd\n");
    assert_eq!(db.count(), 1);
}

#[test]
fn load_missing_file_is_error() {
    assert!(SignatureDb::load(Path::new("/definitely/not/here.txt")).is_err());
}

#[test]
fn scan_bytes_flags_known_hash() {
    let db = db_with("db_hit", ABC_SHA256);
    let path = Path::new("virtual/abc.bin");

    let threats = hash::scan_bytes(path, b"abc", &db).unwrap();
    assert_eq!(threats.len(), 1);
    let t = &threats[0];
    assert_eq!(t.path, path);
    assert_eq!(t.name, format!("known-malware:{}", &ABC_SHA256[..16]));
    assert_eq!(t.matched_rule, "hash-db");
    assert!(matches!(t.severity, Severity::High));

    assert!(hash::scan_bytes(path, b"abd", &db).unwrap().is_empty());
}

#[test]
fn scan_file_matches_scan_bytes() {
    let db = db_with("db_file", ABC_SHA256);
    let hit = temp_file("hit", b"abc");
    let miss = temp_file("miss", b"clean");

    assert_eq!(hash::scan_file(&hit, &db).unwrap().len(), 1);
    assert!(hash::scan_file(&miss, &db).unwrap().is_empty());
    assert!(hash::scan_file(Path::new("/definitely/not/here"), &db).is_err());

    std::fs::remove_file(hit).ok();
    std::fs::remove_file(miss).ok();
}
