use super::*;

#[test]
fn accepts_an_ip_or_hostname_but_not_target_expressions() {
    assert!(is_valid_target("192.168.1.10"));
    assert!(is_valid_target("scan.example.org"));
    assert!(!is_valid_target("192.168.1.0/24"));
    assert!(!is_valid_target("scanme.nmap.org --script vuln"));
}

#[test]
fn parses_open_ports() {
    let xml = r#"<!DOCTYPE nmaprun SYSTEM "nmap.dtd"><nmaprun><host><status state="up"/><address addr="192.0.2.10" addrtype="ipv4"/><ports><port protocol="tcp" portid="443"><state state="open"/><service name="https" product="nginx" version="1.24"/></port><port protocol="tcp" portid="80"><state state="closed"/></port></ports></host></nmaprun>"#;
    let hosts = parse_nmap_xml(xml).unwrap();
    assert_eq!(hosts.len(), 1);
    assert_eq!(hosts[0].ports.len(), 1);
    assert_eq!(hosts[0].ports[0].service, "https");
}

#[test]
fn adds_pn_only_when_requested() {
    let without_pn = ScanRequest {
        target: "192.0.2.1".to_string(),
        profile: "quick".to_string(),
        skip_host_discovery: false,
        timing: "t4".to_string(),
        version_detection: false,
        os_detection: false,
        default_scripts: false,
        ports: String::new(),
        scan_type: "auto".to_string(),
        executor: "local".to_string(),
        exegol_container: String::new(),
    };
    assert!(!scan_args(&without_pn).unwrap().contains(&"-Pn".to_string()));
    let with_pn = ScanRequest {
        skip_host_discovery: true,
        ..without_pn
    };
    assert!(scan_args(&with_pn).unwrap().contains(&"-Pn".to_string()));
}

#[test]
fn accepts_only_valid_custom_port_lists() {
    assert!(is_valid_port_spec("22,80,443,8000-8100"));
    assert!(!is_valid_port_spec("80; -Pn"));
    assert!(!is_valid_port_spec("0"));
}

#[test]
fn custom_ports_replace_the_profile_port_selection() {
    let request = ScanRequest {
        target: "192.0.2.1".to_string(),
        profile: "quick".to_string(),
        skip_host_discovery: false,
        timing: "t4".to_string(),
        version_detection: false,
        os_detection: false,
        default_scripts: false,
        ports: "22,443".to_string(),
        scan_type: "auto".to_string(),
        executor: "local".to_string(),
        exegol_container: String::new(),
    };
    let args = scan_args(&request).unwrap();
    assert!(args.windows(2).any(|pair| pair == ["-p", "22,443"]));
    assert!(!args.contains(&"-F".to_string()));
}

#[test]
fn adds_the_requested_tcp_connect_scan() {
    let request = ScanRequest {
        target: "192.0.2.1".to_string(),
        profile: "service".to_string(),
        skip_host_discovery: false,
        timing: "t4".to_string(),
        version_detection: true,
        os_detection: false,
        default_scripts: false,
        ports: String::new(),
        scan_type: "connect".to_string(),
        executor: "local".to_string(),
        exegol_container: String::new(),
    };
    assert!(scan_args(&request).unwrap().contains(&"-sT".to_string()));
}

#[test]
fn rejects_an_unsafe_container_name() {
    assert!(is_valid_container_name("htb"));
    assert!(!is_valid_container_name("htb; command"));
}

#[test]
fn only_accepts_actions_for_the_matching_service() {
    let request = ServiceProbeRequest {
        target: "192.0.2.10".to_string(),
        port: "22".to_string(),
        service: "ssh".to_string(),
        action: "smtp_capabilities".to_string(),
        executor: "local".to_string(),
        exegol_container: String::new(),
    };
    assert!(service_probe(request).is_err());
}
