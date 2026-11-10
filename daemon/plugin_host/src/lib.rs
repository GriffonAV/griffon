//! Host side of the plugin contract: loads a plugin `.so`, reads its
//! metadata and forwards calls to it.
//!
//! Shared by `griffon-daemon-runner` and the plugin contract tests, so the
//! tests exercise exactly the code path the daemon uses.

use abi_stable::library::lib_header_from_path;
use abi_stable::std_types::{RResult, RString, Tuple2};
use plugin_interface::{PluginRef, PluginRoot_Ref};
use std::path::{Path, PathBuf};

pub struct LoadedPlugin {
    root: PluginRoot_Ref,
    path: PathBuf,
}

/// Metadata returned by a plugin's `init()`.
#[derive(Debug, Default, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub uuid: String,
    pub functions: Vec<String>,
    /// Every raw key/value pair, including the ones above.
    pub entries: Vec<(String, String)>,
}

pub fn is_shared_library(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext| ext == "so")
}

pub fn parse_functions(s: &str) -> Vec<String> {
    s.split(['/', ','])
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

/// - args vide  => "fn:ping"
/// - args [2]   => "fn:ping 2"
/// - args [a,b] => "fn:scan_file a b"
pub fn call_to_wire(fn_name: &str, args: &[String]) -> String {
    // this part will change in the v2
    if args.is_empty() {
        format!("fn:{fn_name}")
    } else {
        format!("fn:{} {}", fn_name, args.join(" "))
    }
}

impl LoadedPlugin {
    pub fn load(path: &Path) -> Result<Self, String> {
        let header = lib_header_from_path(path).map_err(|e| format!("header load failed: {e}"))?;

        let root: PluginRoot_Ref = header
            .init_root_module::<PluginRoot_Ref>()
            .map_err(|e| format!("init root failed: {e}"))?;

        Ok(Self {
            root,
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn init(&self) -> Result<PluginInfo, String> {
        let plugin_ref: &PluginRef = &self.root.plugin();
        let init_fn = plugin_ref.init().ok_or("init() is None")?;

        match init_fn() {
            RResult::ROk(vec) => {
                let mut info = PluginInfo::default();
                for Tuple2(key, value) in vec {
                    match key.as_str() {
                        "function" => info.functions = parse_functions(value.as_str()),
                        "name" => info.name = value.to_string(),
                        "UUID" => info.uuid = value.to_string(),
                        _ => {}
                    }
                    info.entries.push((key.into_string(), value.into_string()));
                }
                Ok(info)
            }
            RResult::RErr(err_msg) => Err(err_msg.into_string()),
        }
    }

    pub fn call(&self, fn_name: &str, args: &[String]) -> Result<String, String> {
        let plugin_ref: &PluginRef = &self.root.plugin();
        let handle_fn = plugin_ref
            .handle_message()
            .ok_or("plugin handle_message() is None")?;

        let response: RString = handle_fn(RString::from(call_to_wire(fn_name, args)));
        Ok(response.into_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_functions_accepts_slash_and_comma() {
        assert_eq!(
            parse_functions(" scan/stop , check//"),
            vec!["scan", "stop", "check"]
        );
    }

    #[test]
    fn call_to_wire_formats_args() {
        assert_eq!(call_to_wire("ping", &[]), "fn:ping");
        assert_eq!(
            call_to_wire("scan_file", &["a".into(), "b".into()]),
            "fn:scan_file a b"
        );
    }
}
