use super::*;
use serde_json::json;
use std::os::unix::process::ExitStatusExt;
use std::process::{ExitStatus, Output};

fn output(code: i32, stdout: &str, stderr: &str) -> std::io::Result<Output> {
    Ok(Output {
        status: ExitStatus::from_raw(code << 8),
        stdout: stdout.as_bytes().to_vec(),
        stderr: stderr.as_bytes().to_vec(),
    })
}
fn request() -> ScanRequest {
    serde_json::from_value(json!({"target":"192.0.2.1","profile":"quick"})).unwrap()
}
fn args(command: &Command) -> Vec<String> {
    command
        .get_args()
        .map(|x| x.to_string_lossy().into_owned())
        .collect()
}
const XML: &str = r#"<nmaprun><host><status state="up"/><address addr="192.0.2.1" addrtype="ipv4"/><hostnames><hostname name="fixture.test"/></hostnames><ports><port protocol="tcp" portid="443"><state state="open"/><service name="https" product="fixture" version="1" extrainfo="TLS"/></port></ports></host><runstats><hosts up="1" total="1"/></runstats></nmaprun>"#;

#[test]
fn target_and_port_boundaries() {
    for s in ["::1", "2001:db8::1", "localhost", "a-b.test"] {
        assert!(is_valid_target(s), "{s}");
    }
    for s in [
        "",
        "-foo",
        "foo-",
        "a..b",
        "a_b",
        "a\nb",
        "é.test",
        "1.2.3.4/24",
    ] {
        assert!(!is_valid_target(s), "{s}");
    }
    assert!(is_valid_target(&"a".repeat(63)));
    assert!(!is_valid_target(&"a".repeat(64)));
    assert!(!is_valid_target(&"a.".repeat(128)));
    for s in ["", "1", "65535", "1-65535", "22,80-80"] {
        assert!(is_valid_port_spec(s));
    }
    for s in ["0", "65536", "80-22", "1-2-3", "1,", "-1", "a", "1, 2"] {
        assert!(!is_valid_port_spec(s), "{s}");
    }
    assert!(!is_valid_container_name(""));
    assert!(!is_valid_container_name(&"a".repeat(129)));
}

#[test]
fn profiles_options_and_invalid_combinations() {
    for (profile, expected) in [
        ("quick", vec!["-T4", "-F"]),
        ("web", vec!["-T4", "-p", "80,443,8080,8443"]),
        ("full_tcp", vec!["-T4", "-p-"]),
        ("discovery", vec!["-T4", "-sn"]),
        ("udp", vec!["-T4", "-sU"]),
        ("service", vec!["-T4"]),
    ] {
        let mut r = request();
        r.profile = profile.into();
        assert_eq!(scan_args(&r).unwrap(), expected);
    }
    let mut r = request();
    r.timing = "t3".into();
    r.scan_type = "connect".into();
    r.version_detection = true;
    r.os_detection = true;
    r.default_scripts = true;
    r.skip_host_discovery = true;
    assert_eq!(
        scan_args(&r).unwrap(),
        vec!["-T3", "-Pn", "-sT", "-sV", "-O", "-sC", "-F"]
    );
    for (key, value) in [
        ("profile", "bad"),
        ("timing", "bad"),
        ("scan_type", "bad"),
        ("ports", "0"),
    ] {
        let mut v = json!({"target":"192.0.2.1","profile":"quick"});
        v[key] = json!(value);
        assert!(scan_args(&serde_json::from_value(v).unwrap()).is_err());
    }
    r.profile = "discovery".into();
    assert!(scan_args(&r).is_err());
    r = request();
    r.scan_type = "syn".into();
    assert_eq!(scan_args(&r).is_ok(), unsafe { libc::geteuid() == 0 });
}

#[test]
fn commands_keep_targets_in_separate_arguments() {
    let mut r = request();
    let a = scan_args(&r).unwrap();
    let c = nmap_command(&r, &a, &r.target).unwrap();
    assert_eq!(c.get_program(), "nmap");
    assert_eq!(args(&c), vec!["-T4", "-F", "-oX", "-", "--", "192.0.2.1"]);
    r.executor = "exegol".into();
    r.exegol_container = " lab ".into();
    let c = nmap_command(&r, &a, &r.target).unwrap();
    assert_eq!(c.get_program(), "docker");
    assert_eq!(&args(&c)[..3], &["exec", "lab", "nmap"]);
    assert!(displayed_command(&r, &a, &r.target).starts_with("docker exec lab nmap"));
    r.exegol_container = ";bad".into();
    assert!(nmap_command(&r, &a, &r.target).is_err());
    r.executor = "bad".into();
    assert!(nmap_command(&r, &a, &r.target).is_err());
    assert!(probe_command("bad", "", "curl").is_err());
    assert!(probe_command("exegol", ";", "curl").is_err());
}

#[test]
fn xml_fields_missing_nodes_and_invalid_xml() {
    let h = parse_nmap_xml(XML).unwrap();
    assert_eq!(h[0].address, "192.0.2.1");
    assert_eq!(h[0].hostname, "fixture.test");
    assert_eq!(h[0].ports[0].extra_info, "TLS");
    assert_eq!(h[0].ports[0].product, "fixture");
    assert_eq!(h[0].ports[0].version, "1");
    assert!(parse_nmap_xml("<broken").is_err());
    assert!(parse_nmap_xml("<nmaprun/>").unwrap().is_empty());
    let h = parse_nmap_xml("<nmaprun><host/></nmaprun>").unwrap();
    assert_eq!(h[0].address, "");
    assert!(h[0].ports.is_empty());
    assert!(scan_feedback("bad", "", &request()).is_err());
}

#[test]
fn scan_success_is_stored_and_failures_are_not() {
    let mut saved = false;
    let result = scan_with(
        request(),
        &mut |c| {
            assert_eq!(c.get_program(), "nmap");
            output(0, XML, "warning")
        },
        &mut |r, e| {
            assert_eq!(e, "local");
            assert_eq!(r.hosts.len(), 1);
            assert_eq!(r.diagnostics, "warning");
            saved = true;
            Ok(())
        },
    )
    .unwrap();
    assert!(saved && result.ok);
    for (code, stdout, stderr) in [(1, "", "failure"), (1, "", ""), (0, "invalid", "")] {
        assert!(
            scan_with(
                request(),
                &mut |_| output(code, stdout, stderr),
                &mut |_, _| panic!("must not store failed scan")
            )
            .is_err()
        );
    }
    assert!(
        scan_with(
            request(),
            &mut |_| Err(std::io::Error::other("missing executable")),
            &mut |_, _| panic!()
        )
        .err()
        .unwrap()
        .contains("Impossible")
    );
    assert!(
        scan_with(
            request(),
            &mut |_| {
                let mut o = output(0, "", "")?;
                o.stdout = vec![255];
                Ok(o)
            },
            &mut |_, _| panic!()
        )
        .is_err()
    );
    assert!(
        scan_with(request(), &mut |_| output(0, XML, ""), &mut |_, _| Err(
            "storage failed".into()
        ))
        .err()
        .unwrap()
        .contains("storage failed")
    );
    let mut r = request();
    r.target = "--bad".into();
    assert!(scan_with(r, &mut |_| panic!(), &mut |_, _| panic!()).is_err());
}

#[test]
fn history_roundtrip_limit_delete_clear_and_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/history.json");
    assert_eq!(load_history_at(&path).unwrap().version, 1);
    let response = scan_with(
        request(),
        &mut |_| output(0, XML, "diagnostic"),
        &mut |_, _| Ok(()),
    )
    .unwrap();
    for _ in 0..102 {
        store_scan_at(&path, &response, "local").unwrap();
    }
    let h = history_response_at(&path).unwrap();
    assert_eq!(h.scans.len(), 100);
    assert_eq!(h.scans[0].diagnostics, "diagnostic");
    assert!(delete_history_entry_at(&path, "missing").is_err());
    let id = h.scans[0].id.clone();
    assert!(
        delete_history_entry_at(&path, &id)
            .unwrap()
            .scans
            .iter()
            .all(|s| s.id != id)
    );
    assert!(clear_history_at(&path).unwrap().scans.is_empty());
    assert!(load_history_at(&path).unwrap().scans.is_empty());
    fs::write(&path, "broken").unwrap();
    assert!(load_history_at(&path).is_err());
    assert!(load_history_at(dir.path()).is_err());
    assert!(save_history_at(&path.join("child"), &ScanHistory::default()).is_err());
}

#[test]
fn docker_listing_filters_unrelated_and_malformed_rows() {
    let result=list_exegol_containers_with(&mut |c| {assert_eq!(args(c),vec!["ps","--format","{{json .}}"]);output(0,"{\"Image\":\"EXEGOL:test\",\"Names\":\"lab\",\"Status\":\"Up\"}\n{}\nbad\n{\"Image\":\"other\"}\n{\"Image\":\"custom\",\"Labels\":\"exegol=true\"}","")}).unwrap();
    assert_eq!(result.containers.len(), 2);
    assert_eq!(result.containers[0].name, "lab");
    assert!(
        list_exegol_containers_with(&mut |_| output(0, "", ""))
            .unwrap()
            .message
            .contains("Aucun")
    );
    assert!(list_exegol_containers_with(&mut |_| output(1, "", "denied")).is_err());
    assert!(list_exegol_containers_with(&mut |_| Err(std::io::Error::other("missing"))).is_err());
}

fn http(probe: &str, executor: &str) -> HttpProbeRequest {
    serde_json::from_value(json!({"target":"2001:db8::1","port":"443","protocol":"https","probe":probe,"executor":executor,"exegol_container":"lab"})).unwrap()
}
#[test]
fn http_probes_construct_urls_and_extract_titles() {
    for executor in ["local", "exegol"] {
        for probe in ["title", "headers", "robots"] {
            let r = http_probe_with(http(probe, executor), &mut |c| {
                assert!(
                    args(c)
                        .last()
                        .unwrap()
                        .starts_with("https://[2001:db8::1]:443")
                );
                if probe == "robots" {
                    assert!(args(c).last().unwrap().ends_with("/robots.txt"));
                }
                output(0, " <TITLE class='x'> Fixture </TITLE> ", "")
            })
            .unwrap();
            assert_eq!(
                r.output,
                if probe == "title" {
                    "Fixture"
                } else {
                    "<TITLE class='x'> Fixture </TITLE>"
                }
            );
        }
    }
    for body in ["none", "<title", "<title>unfinished"] {
        assert_eq!(page_title(body), "No HTML title found.");
    }
    for (key, value) in [
        ("target", "bad target"),
        ("port", "0"),
        ("protocol", "ftp"),
        ("probe", "bad"),
        ("executor", "bad"),
        ("exegol_container", ";"),
    ] {
        let mut r = http("title", "exegol");
        match key {
            "target" => r.target = value.into(),
            "port" => r.port = value.into(),
            "protocol" => r.protocol = value.into(),
            "probe" => r.probe = value.into(),
            "executor" => r.executor = value.into(),
            _ => r.exegol_container = value.into(),
        };
        assert!(http_probe_with(r, &mut |_| panic!()).is_err());
    }
    assert!(http_probe_with(http("title", "local"), &mut |_| output(1, "", "denied")).is_err());
    assert!(
        http_probe_with(http("title", "local"), &mut |_| Err(std::io::Error::other(
            "missing"
        )))
        .is_err()
    );
}
fn service(service: &str, action: &str, executor: &str) -> ServiceProbeRequest {
    serde_json::from_value(json!({"target":"192.0.2.1","port":"22","service":service,"action":action,"executor":executor,"exegol_container":"lab"})).unwrap()
}
#[test]
fn service_probes_use_expected_programs_and_handle_failures() {
    for (s, a, p, flag) in [
        ("ssh", "ssh_keys", "ssh-keyscan", "-T"),
        ("ftp", "ftp_anonymous", "curl", "--user"),
        ("smtp", "smtp_capabilities", "nmap", "smtp-commands"),
        ("dns", "dns_identification", "nmap", "dns-nsid"),
        ("smb", "smb_shares", "smbclient", "-N"),
    ] {
        for e in ["local", "exegol"] {
            let r = service_probe_with(service(s, a, e), &mut |c| {
                assert_eq!(c.get_program(), if e == "local" { p } else { "docker" });
                assert!(args(c).contains(&flag.to_string()));
                output(0, "result", "warning")
            })
            .unwrap();
            assert_eq!(r.output, "result");
        }
    }
    let r = service_probe_with(service("ssh", "ssh_keys", "local"), &mut |_| {
        output(0, "", "keys on stderr")
    })
    .unwrap();
    assert_eq!(r.output, "keys on stderr");
    for err in ["", "failure"] {
        assert!(
            service_probe_with(service("ssh", "ssh_keys", "local"), &mut |_| output(
                1, "", err
            ))
            .is_err()
        );
    }
    assert!(
        service_probe_with(service("ssh", "ssh_keys", "local"), &mut |_| Err(
            std::io::Error::other("missing")
        ))
        .is_err()
    );
    let mut r = service("ssh", "ssh_keys", "local");
    r.port = "0".into();
    assert!(service_probe_with(r, &mut |_| panic!()).is_err());
}
#[test]
fn message_dispatch_rejects_invalid_payloads_without_execution() {
    assert_eq!(split_message(" fn:scan  {} "), ("scan", "{}"));
    assert_eq!(split_message("ping"), ("ping", ""));
    for msg in [
        "unknown",
        "scan {}",
        "http_probe {}",
        "service_probe {}",
        "delete_history_entry {}",
    ] {
        let value: serde_json::Value =
            serde_json::from_str(handle_message(msg.into()).as_str()).unwrap();
        assert_eq!(value["ok"], false);
    }
    assert!(init().is_ok());
    let _ = get_library();
}
