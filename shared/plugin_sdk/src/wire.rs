pub fn split_message(raw: &str) -> (String, String) {
    let raw = raw.trim();

    if raw.starts_with('{')
        && let Ok(json_val) = serde_json::from_str::<serde_json::Value>(raw)
        && let Some(obj) = json_val.as_object()
        && let Some(method_val) = obj.get("method").and_then(|m| m.as_str())
    {
        let params_str = obj
            .get("params")
            .map(|p| p.to_string())
            .unwrap_or_else(|| "null".to_string());
        return (method_val.to_string(), params_str);
    }

    // Fallback to the classic format: "fn:<method> <params>" or "<method> <params>"
    let (head, rest) = match raw.split_once(char::is_whitespace) {
        Some((h, r)) => (h, r.trim()),
        None => (raw, ""),
    };
    let name = head.strip_prefix("fn:").unwrap_or(head);
    (name.to_string(), rest.to_string())
}
pub fn parse_payload(raw_payload: &str) -> Result<serde_json::Value, String> {
    if raw_payload.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(raw_payload).map_err(|e| format!("invalid payload json: {e}"))
}

// --- Les tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_message_with_args() {
        let raw = r#"scan {"path": "/tmp"}"#;
        let (method, payload) = split_message(raw);
        assert_eq!(method, "scan");
        assert_eq!(payload, r#"{"path": "/tmp"}"#);
    }

    #[test]
    fn test_split_message_no_args() {
        let raw = "stop";
        let (method, payload) = split_message(raw);
        assert_eq!(method, "stop");
        assert_eq!(payload, "");
    }
}
