pub fn split_message(raw: &str) -> (String, String) {
    let raw = raw.trim();
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
