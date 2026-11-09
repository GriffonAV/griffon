use super::*;
use crate::unit_tests::context;
fn fixture() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::create_dir(d.path().join("nested")).unwrap();
    fs::write(d.path().join("a.LOG"), b"123").unwrap();
    fs::write(d.path().join("nested/b.log"), b"12345").unwrap();
    fs::write(d.path().join("keep.json"), b"12").unwrap();
    d
}
#[test]
fn clean_counts_and_filters_preserve_other_files() {
    for dry in [true, false] {
        let d = fixture();
        let mut ctx = context(dry);
        ctx.filters.file_types = vec!["log".into()];
        let mut report = ModuleReport::empty("cache");
        CacheCleaner::new()
            .clean_cache_dir("fixture", d.path(), &ctx, &mut report)
            .unwrap();
        assert_eq!(report.bytes_freed, 8);
        assert_eq!(report.files_touched, 2);
        assert_eq!(report.candidate_files_count, 2);
        assert_eq!(report.deleted_files_count, if dry { 0 } else { 2 });
        assert_eq!(report.skipped_files_count, if dry { 2 } else { 0 });
        assert_eq!(report.per_root_path["fixture"].bytes_freed, 8);
        assert_eq!(report.per_file_type["log"].files_touched, 2);
        assert_eq!(d.path().join("a.LOG").exists(), dry);
        assert!(d.path().join("keep.json").exists());
        CacheCleaner::new()
            .clean_cache_dir("missing", &d.path().join("missing"), &ctx, &mut report)
            .unwrap();
        assert_eq!(report.missing_paths_count, 1);
    }
}
#[test]
fn deletion_scope_rejects_root_outside_symlink_and_traversal() {
    let d = fixture();
    let other = fixture();
    let roots = vec![d.path().to_path_buf()];
    for p in [
        PathBuf::new(),
        d.path().join("missing"),
        PathBuf::from("/"),
        d.path().to_path_buf(),
        other.path().join("a.LOG"),
    ] {
        assert!(
            CacheCleaner::validate_delete_target(&p, &roots, false).is_err(),
            "{p:?}"
        );
    }
    assert!(CacheCleaner::validate_delete_target(&d.path().join("a.LOG"), &[], false).is_err());
    assert!(CacheCleaner::validate_delete_target(d.path(), &roots, true).is_ok());
    std::os::unix::fs::symlink(other.path(), d.path().join("link")).unwrap();
    assert!(CacheCleaner::validate_delete_target(&d.path().join("link"), &roots, true).is_err());
    assert!(
        CacheCleaner::validate_delete_target(&d.path().join("link/a.LOG"), &roots, true).is_err()
    );
    assert!(CacheCleaner::validate_delete_target(
        &d.path()
            .join("../")
            .join(other.path().file_name().unwrap())
            .join("a.LOG"),
        &roots,
        true
    )
    .is_err());
}
#[test]
fn selected_deletion_handles_files_directories_filters_and_partial_failure() {
    for dry in [true, false] {
        for filtered in [true, false] {
            let d = fixture();
            let mut ctx = context(dry);
            if filtered {
                ctx.filters.file_types = vec!["log".into()];
            }
            let items = vec![
                d.path().join("a.LOG").display().to_string(),
                d.path().join("nested").display().to_string(),
                d.path().join("missing").display().to_string(),
            ];
            let r = CacheCleaner::new().delete_selected_in(
                &ctx,
                &items,
                false,
                &[d.path().to_path_buf()],
            );
            assert!(!r.ok);
            assert_eq!(r.failed.len(), 1);
            assert_eq!(r.deleted_count, 2);
            assert_eq!(r.deleted_bytes, 8);
            assert_eq!(d.path().join("a.LOG").exists(), dry);
            assert!(d.path().join("keep.json").exists());
            assert_eq!(d.path().join("nested").exists(), dry || filtered);
            if filtered {
                let r = CacheCleaner::new().delete_selected_in(
                    &ctx,
                    &[d.path().join("keep.json").display().to_string()],
                    false,
                    &[d.path().to_path_buf()],
                );
                assert_eq!(r.failed.len(), 1);
                assert_eq!(r.deleted_count, 0);
            }
        }
    }
}
#[test]
fn candidates_are_sorted_and_filtered_with_directory_sizes() {
    let d = fixture();
    fs::write(d.path().join("zero"), b"").unwrap();
    for filter in [
        vec![],
        vec!["log".into()],
        vec!["log".into(), "json".into()],
    ] {
        let mut ctx = context(true);
        ctx.filters.file_types = filter;
        let items = CacheCleaner::new()
            .collect_candidates_in(
                &ctx,
                vec![
                    ("fixture".into(), d.path().to_path_buf()),
                    ("missing".into(), d.path().join("absent")),
                ],
            )
            .unwrap();
        assert_eq!(items[0].name, "nested");
        assert_eq!(items[0].size, 5);
        assert!(!items.iter().any(|x| x.name == "zero"));
        assert!(items.windows(2).all(|p| p[0].size >= p[1].size));
    }
    assert_eq!(CacheCleaner::dir_size(d.path()).unwrap(), 10);
    assert!(CacheCleaner::dir_size(&d.path().join("missing")).is_err());
    let (_, _, errors) = CacheCleaner::delete_matching_files_in_dir(
        &d.path().join("missing"),
        &CleanerFilters::default(),
        false,
    );
    assert_eq!(errors.len(), 1);
}
#[test]
fn file_categories_and_profile_selection() {
    for (path, kind) in [
        ("Packages", "apt_index"),
        ("InRelease", "apt_index"),
        ("Translation-en", "apt_index"),
        ("a.LOG", "log"),
        ("README", "no_ext"),
        ("a.", "unknown"),
    ] {
        assert_eq!(CacheCleaner::file_type_key(Path::new(path)), kind);
    }
    let mut ctx = context(true);
    ctx.config.enable_system_cache = true;
    ctx.config.enable_browser_cache = true;
    ctx.config.enable_dev_cache = true;
    ctx.config.enable_package_cache = true;
    ctx.config.enable_desktop_cache = true;
    for profile in [Profile::Safe, Profile::Full] {
        ctx.config.profile = profile;
        let mut r = ModuleReport::empty("cache");
        for (label, _) in CacheCleaner::default_cache_paths_with_logs(&ctx, &mut r) {
            assert_ne!(CacheCleaner::root_label_to_category(&label), "unknown");
        }
    }
    ctx.config.enable_system_cache = false;
    ctx.config.enable_user_cache = false;
    ctx.config.enable_browser_cache = false;
    ctx.config.enable_dev_cache = false;
    ctx.config.enable_package_cache = false;
    ctx.config.enable_desktop_cache = false;
    let c = CacheCleaner::new();
    assert!(c.run(&ctx).unwrap().per_root_path.is_empty());
    assert!(c.collect_cache_candidates(&ctx).unwrap().is_empty());
    assert!(c.delete_selected_paths(&ctx, &[], false).ok);
    assert!(!c.description().is_empty());
}
