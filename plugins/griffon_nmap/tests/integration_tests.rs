#![cfg(unix)]

const _: () = assert!(
    cfg!(debug_assertions),
    "integration fixtures require debug config/history paths"
);

use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

// A fresh process isolates the plugin's OnceLock router, cwd and environment.
// No global environment mutation in the parallel parent test runner.
fn scenario(name: &str) {
    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("child.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "isolated_session", "--ignored", "--nocapture"])
        .current_dir(dir.path())
        .env("GRIFFON_INTEGRATION_CASE", name)
        .env("GRIFFON_INTEGRATION_ROOT", dir.path())
        .env("HOME", dir.path().join("home"))
        .env("PATH", dir.path().join("bin"))
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!(
                "scenario {name} timed out:\n{}",
                fs::read_to_string(&log_path).unwrap()
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(
        status.success(),
        "scenario {name}:\n{}",
        fs::read_to_string(log_path).unwrap()
    );
}

fn fixture_root() -> std::path::PathBuf {
    let root = std::path::PathBuf::from(
        std::env::var_os("GRIFFON_INTEGRATION_ROOT").expect("only run through scenario()"),
    );
    assert_eq!(std::env::current_dir().unwrap(), root);
    assert_eq!(std::env::var_os("HOME").unwrap(), root.join("home"));
    assert_eq!(std::env::var_os("PATH").unwrap(), root.join("bin"));
    fs::create_dir_all(root.join("home")).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    root
}

fn executable(path: &Path, script: &str) {
    fs::write(path, script).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

const XML: &str = r#"<?xml version="1.0"?><!DOCTYPE nmaprun><nmaprun><host><status state="up"/><address addr="192.0.2.42" addrtype="ipv4"/><hostnames><hostname name="fixture.test"/></hostnames><ports><port protocol="tcp" portid="443"><state state="open"/><service name="https" product="fixture-server" version="1.0"/></port><port protocol="tcp" portid="80"><state state="closed"/></port></ports></host><runstats><finished summary="fixture completed"/><hosts up="1" total="1"/></runstats></nmaprun>"#;
const HISTORY: &str = ".data/griffon/nmap/history.json";

fn call(method: &str, params: Value) -> Value {
    let plugin = griffon_nmap::get_library().plugin();
    let response = (plugin
        .handle_message()
        .expect("plugin must export handle_message"))(
        json!({"method":method,"params":params}).to_string().into(),
    );
    serde_json::from_str(response.as_str()).expect("plugin response must be valid JSON")
}
fn request() -> Value {
    json!({"target":"192.0.2.42","profile":"web","scan_type":"connect"})
}
fn fake_process(root: &Path, program: &str, body: &str, stderr: &str, status: u8) {
    fs::write(root.join("stdout.fixture"), body).unwrap();
    fs::write(root.join("stderr.fixture"), stderr).unwrap();
    executable(
        &root.join("bin").join(program),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > command-args.txt\n/bin/cat stdout.fixture\n/bin/cat stderr.fixture >&2\nexit {status}\n"
        ),
    );
}

#[test]
fn scan_xml_response_and_history_roundtrip() {
    scenario("roundtrip");
}
#[test]
fn exegol_command_routes_and_persists_executor() {
    scenario("exegol");
}
#[test]
fn failed_scans_never_create_history() {
    scenario("errors");
}
#[test]
fn down_host_diagnostics_survive_history_reload() {
    scenario("down_host");
}
#[test]
fn history_limit_corruption_and_storage_errors() {
    scenario("history_errors");
}

#[test]
fn http_probes_roundtrip_and_errors() {
    scenario("http_probes");
}
#[test]
fn service_probes_roundtrip_and_errors() {
    scenario("service_probes");
}
#[test]
fn container_listing_filters_and_errors() {
    scenario("container_listing");
}

#[test]
#[ignore = "subprocess entry point; invoked by the integration scenarios"]
fn isolated_session() {
    let root = fixture_root();
    fake_process(&root, "nmap", XML, "fixture warning\n", 0);
    match std::env::var("GRIFFON_INTEGRATION_CASE").unwrap().as_str() {
        "roundtrip" | "exegol" => {
            assert_eq!(call("list_history", json!(null))["scans"], json!([]));
            let exegol = std::env::var("GRIFFON_INTEGRATION_CASE").unwrap() == "exegol";
            let mut req = request();
            if exegol {
                fake_process(&root, "docker", XML, "fixture warning\n", 0);
                req["executor"] = json!("exegol");
                req["exegol_container"] = json!("fixture-lab");
            }
            let response = call("scan", req);
            assert_eq!(response["ok"], true, "{response}");
            assert_eq!(response["hosts"][0]["address"], "192.0.2.42");
            assert_eq!(response["hosts"][0]["hostname"], "fixture.test");
            assert_eq!(response["hosts"][0]["ports"].as_array().unwrap().len(), 1);
            assert_eq!(response["hosts"][0]["ports"][0]["service"], "https");
            assert_eq!(
                response["diagnostics"],
                "fixture warning\nfixture completed"
            );
            let args = fs::read_to_string(root.join("command-args.txt")).unwrap();
            let expected = if exegol {
                "exec\nfixture-lab\nnmap\n"
            } else {
                ""
            };
            assert_eq!(
                args,
                format!("{expected}-T4\n-sT\n-p\n80,443,8080,8443\n-oX\n-\n--\n192.0.2.42\n")
            );
            let history = call("list_history", json!({}));
            assert_eq!(history["scans"].as_array().unwrap().len(), 1);
            let entry = &history["scans"][0];
            assert_eq!(entry["hosts"], response["hosts"]);
            assert_eq!(entry["diagnostics"], response["diagnostics"]);
            assert_eq!(entry["executor"], if exegol { "exegol" } else { "local" });
            let disk: Value =
                serde_json::from_str(&fs::read_to_string(root.join(HISTORY)).unwrap()).unwrap();
            assert_eq!(disk["version"], 1);
            assert_eq!(disk["scans"], history["scans"]);
            assert_eq!(
                call("delete_history_entry", json!({"id":"absent"}))["ok"],
                false
            );
            assert_eq!(call("list_history", json!(null))["scans"], history["scans"]);
            assert_eq!(
                call("delete_history_entry", json!({"id":entry["id"]}))["scans"],
                json!([])
            );
            assert_eq!(call("scan", request())["ok"], true);
            assert_eq!(call("clear_history", json!(null))["scans"], json!([]));
            assert_eq!(call("list_history", json!(null))["scans"], json!([]));
        }
        "errors" => {
            assert_eq!(
                call("scan", json!({"target":"--invalid","profile":"quick"}))["ok"],
                false
            );
            assert!(!root.join("command-args.txt").exists());
            for (body, stderr, status, expected) in [
                ("", "fixture failure", 2, "fixture failure"),
                ("broken XML", "", 0, "XML invalide"),
            ] {
                fake_process(&root, "nmap", body, stderr, status);
                let response = call("scan", request());
                assert_eq!(response["ok"], false);
                assert!(
                    response["message"].as_str().unwrap().contains(expected),
                    "{response}"
                );
                assert!(!root.join(HISTORY).exists());
            }
            fs::write(root.join("stdout.fixture"), [255]).unwrap();
            assert!(
                call("scan", request())["message"]
                    .as_str()
                    .unwrap()
                    .contains("UTF-8")
            );
            fs::remove_file(root.join("bin/nmap")).unwrap();
            assert!(
                call("scan", request())["message"]
                    .as_str()
                    .unwrap()
                    .contains("Impossible de lancer Nmap")
            );
            assert_eq!(call("list_history", json!(null))["scans"], json!([]));
            assert!(!root.join(HISTORY).exists());
        }
        "down_host" => {
            fake_process(
                &root,
                "nmap",
                r#"<nmaprun><runstats><finished summary="no active hosts"/><hosts up="0" total="1"/></runstats></nmaprun>"#,
                "discovery warning",
                0,
            );
            let response = call("scan", request());
            assert_eq!(response["ok"], true);
            assert_eq!(response["hosts"], json!([]));
            assert!(response["message"].as_str().unwrap().contains("-Pn"));
            let history = call("list_history", json!({}));
            assert_eq!(history["scans"][0]["message"], response["message"]);
            assert_eq!(
                history["scans"][0]["diagnostics"],
                "discovery warning\nno active hosts"
            );
        }
        "history_errors" => {
            assert_eq!(call("scan", request())["ok"], true);
            let path = root.join(HISTORY);
            let mut disk: Value =
                serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
            let template = disk["scans"][0].clone();
            disk["scans"] = Value::Array(
                (0..100)
                    .map(|i| {
                        let mut e = template.clone();
                        e["id"] = json!(format!("old-{i}"));
                        e
                    })
                    .collect(),
            );
            fs::write(&path, disk.to_string()).unwrap();
            assert_eq!(call("scan", request())["ok"], true);
            let history = call("list_history", json!(null));
            let entries = history["scans"].as_array().unwrap();
            assert_eq!(entries.len(), 100);
            assert!(entries[0]["id"].as_str().unwrap().starts_with("scan-"));
            assert_eq!(entries[1]["id"], "old-0");
            assert_eq!(entries[99]["id"], "old-98");
            fs::write(&path, "corrupt").unwrap();
            assert_eq!(call("list_history", json!(null))["ok"], false);
            assert_eq!(call("scan", request())["ok"], false);
            assert_eq!(fs::read_to_string(&path).unwrap(), "corrupt");
            fs::remove_file(&path).unwrap();
            // Deterministic write failure even when run as root: temp output is a directory.
            fs::create_dir(path.with_extension("json.tmp")).unwrap();
            let response = call("scan", request());
            assert_eq!(response["ok"], false);
            assert!(
                response["message"]
                    .as_str()
                    .unwrap()
                    .contains("écrire l’historique")
            );
            assert!(!path.exists());
        }
        "http_probes" => {
            assert_eq!(call("scan", request())["ok"], true);
            let saved = fs::read(root.join(HISTORY)).unwrap();
            for executor in ["local", "exegol"] {
                for (probe, body, expected, suffix) in [
                    ("title", "<TITLE>Fixture title</TITLE>", "Fixture title", ""),
                    (
                        "headers",
                        "HTTP/1.1 200 OK\nServer: fixture\n",
                        "HTTP/1.1 200 OK\nServer: fixture",
                        "",
                    ),
                    (
                        "robots",
                        "User-agent: *\nDisallow: /private\n",
                        "User-agent: *\nDisallow: /private",
                        "/robots.txt",
                    ),
                ] {
                    fake_process(
                        &root,
                        if executor == "local" {
                            "curl"
                        } else {
                            "docker"
                        },
                        body,
                        "",
                        0,
                    );
                    let response = call(
                        "http_probe",
                        json!({"target":"2001:db8::42","port":"8443","protocol":"https","probe":probe,"executor":executor,"exegol_container":"lab"}),
                    );
                    assert_eq!(response["ok"], true, "{response}");
                    assert_eq!(response["output"], expected);
                    assert_eq!(response["probe"], probe);
                    let prefix = if executor == "exegol" {
                        "exec\nlab\ncurl\n"
                    } else {
                        ""
                    };
                    let headers = if probe == "headers" { "-I\n" } else { "" };
                    assert_eq!(
                        fs::read_to_string(root.join("command-args.txt")).unwrap(),
                        format!(
                            "{prefix}-k\n--max-time\n15\n{headers}-sS\nhttps://[2001:db8::42]:8443{suffix}\n"
                        )
                    );
                }
            }
            let req =
                json!({"target":"fixture.test","port":"443","protocol":"https","probe":"title"});
            fake_process(&root, "curl", "", "connection refused", 7);
            assert!(
                call("http_probe", req.clone())["message"]
                    .as_str()
                    .unwrap()
                    .contains("connection refused")
            );
            let last_command = fs::read(root.join("command-args.txt")).unwrap();
            let mut invalid = req.clone();
            invalid["protocol"] = json!("file");
            assert_eq!(call("http_probe", invalid)["ok"], false);
            assert_eq!(
                fs::read(root.join("command-args.txt")).unwrap(),
                last_command
            );
            fs::remove_file(root.join("bin/curl")).unwrap();
            assert!(
                call("http_probe", req)["message"]
                    .as_str()
                    .unwrap()
                    .contains("Impossible de lancer curl")
            );
            assert_eq!(fs::read(root.join(HISTORY)).unwrap(), saved);
        }
        "service_probes" => {
            assert_eq!(call("scan", request())["ok"], true);
            let saved = fs::read(root.join(HISTORY)).unwrap();
            for (service, action, program, port, args) in [
                (
                    "ssh",
                    "ssh_keys",
                    "ssh-keyscan",
                    "22",
                    "-T\n10\n-p\n22\n192.0.2.42\n",
                ),
                (
                    "ftp",
                    "ftp_anonymous",
                    "curl",
                    "21",
                    "-sS\n--max-time\n15\n--user\nanonymous:anonymous\nftp://192.0.2.42:21/\n",
                ),
                (
                    "smtp",
                    "smtp_capabilities",
                    "nmap",
                    "25",
                    "-Pn\n-p\n25\n--script\nsmtp-commands\n-oN\n-\n--\n192.0.2.42\n",
                ),
                (
                    "dns",
                    "dns_identification",
                    "nmap",
                    "53",
                    "-Pn\n-sU\n-p\n53\n--script\ndns-nsid\n-oN\n-\n--\n192.0.2.42\n",
                ),
                (
                    "smb",
                    "smb_shares",
                    "smbclient",
                    "445",
                    "-L\n//192.0.2.42\n-p\n445\n-N\n",
                ),
            ] {
                for executor in ["local", "exegol"] {
                    fake_process(
                        &root,
                        if executor == "local" {
                            program
                        } else {
                            "docker"
                        },
                        "fixture result\n",
                        "warning",
                        0,
                    );
                    let response = call(
                        "service_probe",
                        json!({"target":"192.0.2.42","port":port,"service":service,"action":action,"executor":executor,"exegol_container":"lab"}),
                    );
                    assert_eq!(response["ok"], true, "{response}");
                    assert_eq!(response["output"], "fixture result");
                    assert_eq!(response["action"], action);
                    let prefix = if executor == "exegol" {
                        format!("exec\nlab\n{program}\n")
                    } else {
                        String::new()
                    };
                    assert_eq!(
                        fs::read_to_string(root.join("command-args.txt")).unwrap(),
                        format!("{prefix}{args}")
                    );
                }
            }
            let req =
                json!({"target":"192.0.2.42","port":"22","service":"ssh","action":"ssh_keys"});
            fake_process(&root, "ssh-keyscan", "", "stderr result", 0);
            assert_eq!(
                call("service_probe", req.clone())["output"],
                "stderr result"
            );
            fake_process(&root, "ssh-keyscan", "", "fixture denied", 1);
            assert!(
                call("service_probe", req.clone())["message"]
                    .as_str()
                    .unwrap()
                    .contains("fixture denied")
            );
            let last_command = fs::read(root.join("command-args.txt")).unwrap();
            let mut invalid = req.clone();
            invalid["action"] = json!("smb_shares");
            assert_eq!(call("service_probe", invalid)["ok"], false);
            assert_eq!(
                fs::read(root.join("command-args.txt")).unwrap(),
                last_command
            );
            fs::remove_file(root.join("bin/ssh-keyscan")).unwrap();
            assert_eq!(call("service_probe", req)["ok"], false);
            assert_eq!(fs::read(root.join(HISTORY)).unwrap(), saved);
        }
        "container_listing" => {
            let rows = concat!(
                "{\"Image\":\"exegol:latest\",\"Names\":\"lab-a\",\"Status\":\"Up\"}\n",
                "{\"Image\":\"custom\",\"Labels\":\"org.exegol=true\",\"Names\":\"lab-b\"}\n",
                "{\"Image\":\"postgres\",\"Names\":\"unrelated\"}\n{}\nnot-json\n"
            );
            fake_process(&root, "docker", rows, "", 0);
            let result = call("list_exegol_containers", json!(null));
            assert_eq!(result["ok"], true);
            assert_eq!(result["containers"].as_array().unwrap().len(), 2);
            assert_eq!(result["containers"][0]["name"], "lab-a");
            assert_eq!(result["containers"][1]["name"], "lab-b");
            assert_eq!(
                fs::read_to_string(root.join("command-args.txt")).unwrap(),
                "ps\n--format\n{{json .}}\n"
            );
            fake_process(&root, "docker", "", "", 0);
            assert_eq!(
                call("list_exegol_containers", json!({}))["containers"],
                json!([])
            );
            fake_process(&root, "docker", "", "daemon unavailable", 1);
            assert!(
                call("list_exegol_containers", json!({}))["message"]
                    .as_str()
                    .unwrap()
                    .contains("daemon unavailable")
            );
            fs::remove_file(root.join("bin/docker")).unwrap();
            assert!(
                call("list_exegol_containers", json!({}))["message"]
                    .as_str()
                    .unwrap()
                    .contains("Impossible de lancer Docker")
            );
            assert!(!root.join(HISTORY).exists());
        }
        other => panic!("unknown scenario: {other}"),
    }
}
