//! Client-side expansion of path placeholders in plugin call arguments.
//!
//! The daemon runs as a system service (root in installed builds), so it
//! cannot resolve per-user folders such as Downloads. The GUI runs as the
//! user and replaces placeholders with real paths before sending a call.

use serde_json::Value;
use std::path::PathBuf;

/// Placeholder for the user's Downloads folder, optionally followed by a
/// sub-path (`##/download/sub/dir`).
pub const DOWNLOAD_PLACEHOLDER: &str = "##/download";

/// Replaces `##/download` placeholders in `args`.
///
/// Each argument is either a JSON document (placeholders are replaced in every
/// string value) or a plain string. Arguments without placeholders are passed
/// through unchanged. `download_dir` is only called when a placeholder is
/// present.
pub fn expand_path_placeholders(
    args: Vec<String>,
    download_dir: impl FnOnce() -> Option<PathBuf>,
) -> Result<Vec<String>, String> {
    if !args.iter().any(|arg| arg.contains(DOWNLOAD_PLACEHOLDER)) {
        return Ok(args);
    }

    let dir = download_dir().ok_or("Could not determine your Downloads folder")?;
    let dir = dir
        .to_str()
        .ok_or_else(|| {
            format!(
                "Downloads folder path is not valid UTF-8: {}",
                dir.display()
            )
        })?
        .to_string();

    args.into_iter()
        .map(|arg| {
            if !arg.contains(DOWNLOAD_PLACEHOLDER) {
                return Ok(arg);
            }
            match serde_json::from_str::<Value>(&arg) {
                Ok(mut value) => {
                    replace_in_value(&mut value, &dir);
                    serde_json::to_string(&value).map_err(|e| e.to_string())
                }
                Err(_) => Ok(replace_in_str(&arg, &dir).unwrap_or(arg)),
            }
        })
        .collect()
}

fn replace_in_value(value: &mut Value, dir: &str) {
    match value {
        Value::String(s) => {
            if let Some(expanded) = replace_in_str(s, dir) {
                *s = expanded;
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|v| replace_in_value(v, dir)),
        Value::Object(map) => map.values_mut().for_each(|v| replace_in_value(v, dir)),
        _ => {}
    }
}

/// Expands `s` if it is exactly the placeholder or starts with `placeholder/`.
fn replace_in_str(s: &str, dir: &str) -> Option<String> {
    let rest = s.strip_prefix(DOWNLOAD_PLACEHOLDER)?;
    if rest.is_empty() || rest.starts_with('/') {
        Some(format!("{dir}{rest}"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn downloads() -> Option<PathBuf> {
        Some(PathBuf::from("/home/user/Downloads"))
    }

    #[test]
    fn expands_placeholder_in_json_settings() {
        let arg = json!({ "paths": ["##/download"], "folder": true }).to_string();
        let out = expand_path_placeholders(vec![arg], downloads).unwrap();
        let value: Value = serde_json::from_str(&out[0]).unwrap();
        assert_eq!(value["paths"], json!(["/home/user/Downloads"]));
        assert_eq!(value["folder"], json!(true));
    }

    #[test]
    fn expands_sub_paths_and_plain_strings() {
        let out = expand_path_placeholders(
            vec!["##/download/sub".into(), "##/download".into()],
            downloads,
        )
        .unwrap();
        assert_eq!(out, ["/home/user/Downloads/sub", "/home/user/Downloads"]);
    }

    #[test]
    fn leaves_other_values_untouched() {
        let args = vec![
            json!({ "paths": ["/srv/data", "##/downloads-not-a-placeholder"] }).to_string(),
            "{not json".into(),
        ];
        let out = expand_path_placeholders(args.clone(), downloads).unwrap();
        let value: Value = serde_json::from_str(&out[0]).unwrap();
        assert_eq!(
            value["paths"],
            json!(["/srv/data", "##/downloads-not-a-placeholder"])
        );
        assert_eq!(out[1], args[1]);
    }

    #[test]
    fn does_not_resolve_downloads_without_placeholder() {
        let args = vec![json!({ "paths": ["/srv/data"] }).to_string()];
        let out = expand_path_placeholders(args.clone(), || panic!("must not be called")).unwrap();
        assert_eq!(out, args);
    }

    #[test]
    fn reports_missing_downloads_folder() {
        let err = expand_path_placeholders(vec!["##/download".into()], || None).unwrap_err();
        assert!(err.contains("Downloads"), "{err}");
    }
}
