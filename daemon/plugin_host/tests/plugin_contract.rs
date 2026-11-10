//! Contract tests: every first-party plugin must load through the same code
//! path as `griffon-daemon-runner` and agree with its manifest.
//!
//! These only check the host <-> plugin boundary (load, init, call), not what
//! the plugin functions actually do; that belongs in each plugin's own tests.

use plugin_host::LoadedPlugin;
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// Cargo package name and manifest path (relative to the workspace root).
const PLUGINS: &[(&str, &str)] = &[
    (
        "griffon_scanner",
        "plugins/griffon_scanner/libgriffon_scanner.toml",
    ),
    (
        "griffon_cleaner",
        "plugins/griffon_cleaner/libgriffon_cleaner.toml",
    ),
    ("griffon_nmap", "plugins/griffon_nmap/libgriffon_nmap.toml"),
    (
        "docker_helper",
        "plugins/docker_helper/libdocker_helper.toml",
    ),
    (
        "plugin_template",
        "plugins/plugin_template/plugin_template.toml",
    ),
];

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Builds every plugin cdylib once and returns `package name -> .so path`.
///
/// `cargo test` does not build the `.so` of other crates, so we ask cargo
/// for them and read the artifact paths from its JSON output.
fn built_plugins() -> &'static HashMap<String, PathBuf> {
    static BUILT: OnceLock<HashMap<String, PathBuf>> = OnceLock::new();
    BUILT.get_or_init(|| {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let mut cmd = Command::new(cargo);
        cmd.current_dir(workspace_root())
            .args(["build", "--message-format=json-render-diagnostics"]);
        for (package, _) in PLUGINS {
            cmd.args(["-p", package]);
        }

        let output = cmd.output().expect("failed to run cargo build");
        assert!(output.status.success(), "building plugins failed");

        let mut built = HashMap::new();
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            if msg["reason"] != "compiler-artifact" {
                continue;
            }
            let Some(so) = msg["filenames"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|f| f.as_str())
                .find(|f| f.ends_with(".so"))
            else {
                continue;
            };
            let package = msg["target"]["name"].as_str().unwrap_or_default();
            built.insert(package.to_string(), PathBuf::from(so));
        }
        built
    })
}

fn load_manifest(manifest: &str) -> toml::Table {
    let path = workspace_root().join(manifest);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    toml::from_str(&content).unwrap_or_else(|e| panic!("invalid manifest {manifest}: {e}"))
}

/// Every `fn = "..."` referenced by the manifest's interactions.
fn manifest_functions(manifest: &toml::Table) -> BTreeSet<String> {
    manifest
        .get("interactions")
        .and_then(|i| i.as_array())
        .into_iter()
        .flatten()
        .filter_map(|interaction| interaction.get("steps")?.as_array())
        .flatten()
        .filter_map(|step| step.get("fn")?.as_str().map(String::from))
        .collect()
}

fn check_contract(package: &str, manifest_path: &str) {
    let so = built_plugins()
        .get(package)
        .unwrap_or_else(|| panic!("no .so built for {package}"));

    let plugin = LoadedPlugin::load(so).unwrap_or_else(|e| panic!("{package}: {e}"));
    let info = plugin
        .init()
        .unwrap_or_else(|e| panic!("{package}: init() failed: {e}"));

    assert!(
        !info.name.trim().is_empty(),
        "{package}: init() has no name"
    );
    assert!(
        !info.functions.is_empty(),
        "{package}: init() declares no functions"
    );

    let uuid = uuid::Uuid::parse_str(&info.uuid)
        .unwrap_or_else(|e| panic!("{package}: invalid UUID {:?}: {e}", info.uuid));
    assert!(!uuid.is_nil(), "{package}: UUID is nil");

    let manifest = load_manifest(manifest_path);
    let header = manifest["plugin"]
        .as_table()
        .unwrap_or_else(|| panic!("{manifest_path}: missing [plugin]"));
    for key in ["uuid", "id"] {
        let value = header
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("{manifest_path}: missing plugin.{key}"));
        assert_eq!(
            uuid::Uuid::parse_str(value).ok(),
            Some(uuid),
            "{package}: plugin.{key} in {manifest_path} does not match init() UUID"
        );
    }

    let declared: BTreeSet<String> = info.functions.iter().cloned().collect();
    let missing: Vec<_> = manifest_functions(&manifest)
        .difference(&declared)
        .cloned()
        .collect();
    assert!(
        missing.is_empty(),
        "{package}: {manifest_path} calls functions not declared by init(): {missing:?}"
    );

    // A panic inside the plugin aborts the process, so returning at all is the check.
    plugin
        .call("__contract_unknown_function__", &[])
        .unwrap_or_else(|e| panic!("{package}: call failed: {e}"));
}

macro_rules! contract_tests {
    ($($test:ident => $package:literal),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                let (_, manifest) = PLUGINS
                    .iter()
                    .find(|(p, _)| *p == $package)
                    .expect("plugin missing from PLUGINS");
                check_contract($package, manifest);
            }
        )*
    };
}

contract_tests! {
    scanner_contract => "griffon_scanner",
    cleaner_contract => "griffon_cleaner",
    nmap_contract => "griffon_nmap",
    docker_helper_contract => "docker_helper",
    plugin_template_contract => "plugin_template",
}
