use super::*;

fn request() -> ScanRequest {
    serde_json::from_str(r#"{"target":"stream.htb","profile":"full_tcp"}"#).unwrap()
}

#[test]
fn down_host_has_feedback_even_without_host_nodes() {
    let xml = r#"<!DOCTYPE nmaprun><nmaprun><runstats><finished summary="Nmap done: 1 IP address (0 hosts up) scanned in 2.24 seconds"/><hosts up="0" down="1" total="1"/></runstats></nmaprun>"#;
    assert!(parse_nmap_xml(xml).unwrap().is_empty());
    let (message, diagnostics) = scan_feedback(xml, "Warning from Nmap\n", &request()).unwrap();
    assert!(message.contains("Aucun hôte détecté actif"));
    assert!(message.contains("-Pn"));
    assert_eq!(
        diagnostics,
        "Warning from Nmap\nNmap done: 1 IP address (0 hosts up) scanned in 2.24 seconds"
    );
    let mut request = request();
    request.skip_host_discovery = true;
    assert!(!scan_feedback(xml, "", &request).unwrap().0.contains("-Pn"));
    request.skip_host_discovery = false;
    request.profile = "discovery".to_string();
    assert!(!scan_feedback(xml, "", &request).unwrap().0.contains("-Pn"));
}

#[test]
fn no_target_is_distinct_from_a_down_host() {
    let xml = r#"<nmaprun><runstats><hosts up="0" total="0"/></runstats></nmaprun>"#;
    let (message, diagnostics) =
        scan_feedback(xml, "Failed to resolve target", &request()).unwrap();
    assert!(message.contains("résolution DNS"));
    assert!(!message.contains("-Pn"));
    assert_eq!(diagnostics, "Failed to resolve target");
}

#[test]
fn active_host_without_open_ports_is_not_reported_down() {
    let xml = r#"<nmaprun><host><status state="up"/></host><runstats><hosts up="1" total="1"/></runstats></nmaprun>"#;
    assert_eq!(
        scan_feedback(xml, "", &request()).unwrap().0,
        "Analyse terminée."
    );
}

#[test]
fn old_history_remains_readable() {
    let entry: HistoryEntry = serde_json::from_str(r#"{"id":"old","created_at_ms":1,"target":"stream.htb","profile":"quick","executor":"local","command":"nmap stream.htb","duration_ms":1,"hosts":[]}"#).unwrap();
    assert!(entry.message.is_empty());
    assert!(entry.diagnostics.is_empty());
}
