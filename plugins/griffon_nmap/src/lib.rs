use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RResult, RString, RVec, Tuple2},
};
use plugin_interface::{PluginI, PluginRoot, PluginRoot_Ref};
use serde::{Deserialize, Serialize};

const MAX_TARGET_LENGTH: usize = 253;

#[derive(Deserialize)]
struct ScanRequest {
    target: String,
    profile: String,
    #[serde(default)]
    skip_host_discovery: bool,
    #[serde(default = "default_timing")]
    timing: String,
    #[serde(default)]
    version_detection: bool,
    #[serde(default)]
    os_detection: bool,
    #[serde(default)]
    default_scripts: bool,
    #[serde(default)]
    ports: String,
    #[serde(default = "default_scan_type")]
    scan_type: String,
    #[serde(default = "default_executor")]
    executor: String,
    #[serde(default)]
    exegol_container: String,
}

fn default_timing() -> String {
    "t4".to_string()
}

fn default_scan_type() -> String {
    "auto".to_string()
}

fn default_executor() -> String {
    "local".to_string()
}

#[derive(Serialize)]
struct ScanResponse {
    ok: bool,
    message: String,
    target: String,
    profile: String,
    command: String,
    duration_ms: u128,
    hosts: Vec<Host>,
}

#[derive(Serialize)]
struct ExegolContainer {
    name: String,
    image: String,
    status: String,
}

#[derive(Serialize)]
struct ExegolContainerList {
    ok: bool,
    message: String,
    containers: Vec<ExegolContainer>,
}

#[derive(Deserialize)]
struct HttpProbeRequest {
    target: String,
    port: String,
    protocol: String,
    probe: String,
    #[serde(default = "default_executor")]
    executor: String,
    #[serde(default)]
    exegol_container: String,
}

#[derive(Serialize)]
struct HttpProbeResponse {
    ok: bool,
    target: String,
    port: String,
    probe: String,
    command: String,
    output: String,
    message: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Host {
    address: String,
    hostname: String,
    state: String,
    ports: Vec<Port>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Port {
    port: String,
    protocol: String,
    state: String,
    service: String,
    product: String,
    version: String,
    extra_info: String,
}

#[derive(Serialize, Deserialize, Default)]
struct ScanHistory {
    version: u8,
    scans: Vec<HistoryEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
struct HistoryEntry {
    id: String,
    created_at_ms: u128,
    target: String,
    profile: String,
    executor: String,
    command: String,
    duration_ms: u128,
    hosts: Vec<Host>,
}

#[derive(Serialize)]
struct HistoryResponse {
    ok: bool,
    message: String,
    scans: Vec<HistoryEntry>,
}

#[derive(Deserialize)]
struct DeleteHistoryRequest {
    id: String,
}

fn json_error(message: impl Into<String>) -> RString {
    RString::from(
        serde_json::json!({ "ok": false, "message": message.into(), "hosts": [] }).to_string(),
    )
}

fn history_path() -> PathBuf {
    if cfg!(debug_assertions) {
        return PathBuf::from(".data/griffon/nmap/history.json");
    }
    PathBuf::from("/var/lib/griffon/nmap/history.json")
}

fn load_history() -> Result<ScanHistory, String> {
    let path = history_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|error| format!("Historique Nmap invalide dans {} : {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(ScanHistory {
            version: 1,
            scans: Vec::new(),
        }),
        Err(error) => Err(format!("Impossible de lire l’historique Nmap : {error}")),
    }
}

fn save_history(history: &ScanHistory) -> Result<(), String> {
    let path = history_path();
    let parent = path.parent().ok_or("Chemin d’historique invalide")?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Impossible de créer le dossier d’historique : {error}"))?;
    let temporary = path.with_extension("json.tmp");
    let contents = serde_json::to_vec_pretty(history)
        .map_err(|error| format!("Impossible de sérialiser l’historique : {error}"))?;
    fs::write(&temporary, contents)
        .map_err(|error| format!("Impossible d’écrire l’historique : {error}"))?;
    fs::rename(&temporary, &path)
        .map_err(|error| format!("Impossible de finaliser l’historique : {error}"))
}

fn store_scan(response: &ScanResponse, executor: &str) -> Result<(), String> {
    let mut history = load_history()?;
    let created_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    history.scans.insert(
        0,
        HistoryEntry {
            id: format!("scan-{created_at_ms}"),
            created_at_ms,
            target: response.target.clone(),
            profile: response.profile.clone(),
            executor: executor.to_string(),
            command: response.command.clone(),
            duration_ms: response.duration_ms,
            hosts: response.hosts.clone(),
        },
    );
    history.scans.truncate(100);
    save_history(&history)
}

fn history_response() -> Result<HistoryResponse, String> {
    let history = load_history()?;
    Ok(HistoryResponse {
        ok: true,
        message: format!("{} analyse(s) enregistrée(s).", history.scans.len()),
        scans: history.scans,
    })
}

fn delete_history_entry(id: &str) -> Result<HistoryResponse, String> {
    let mut history = load_history()?;
    let previous_count = history.scans.len();
    history.scans.retain(|scan| scan.id != id);
    if history.scans.len() == previous_count {
        return Err("Cette analyse n’existe plus dans l’historique.".to_string());
    }
    save_history(&history)?;
    Ok(HistoryResponse {
        ok: true,
        message: "Analyse supprimée de l’historique.".to_string(),
        scans: history.scans,
    })
}

fn clear_history() -> Result<HistoryResponse, String> {
    let mut history = load_history()?;
    history.scans.clear();
    save_history(&history)?;
    Ok(HistoryResponse {
        ok: true,
        message: "Historique Nmap supprimé.".to_string(),
        scans: Vec::new(),
    })
}

fn is_valid_target(target: &str) -> bool {
    if target.is_empty()
        || target.len() > MAX_TARGET_LENGTH
        || target.chars().any(char::is_whitespace)
    {
        return false;
    }
    if target.parse::<IpAddr>().is_ok() {
        return true;
    }

    // A hostname is passed as a distinct process argument, but we still keep the
    // accepted format deliberately narrow and reject Nmap's target expressions.
    target.split('.').all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    })
}

fn is_valid_container_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
}

fn profile_args(profile: &str) -> Result<&'static [&'static str], String> {
    match profile {
        "quick" | "web" | "full_tcp" => Ok(&[]),
        "service" => Ok(&[]),
        "discovery" => Ok(&["-sn"]),
        "udp" => Ok(&["-sU"]),
        _ => Err("Profil Nmap inconnu.".to_string()),
    }
}

fn valid_port_number(value: &str) -> bool {
    value.parse::<u16>().is_ok_and(|port| port > 0)
}

fn is_valid_port_spec(ports: &str) -> bool {
    let ports = ports.trim();
    if ports.is_empty() {
        return true;
    }
    ports
        .split(',')
        .all(|segment| match segment.split_once('-') {
            Some((start, end)) => {
                valid_port_number(start)
                    && valid_port_number(end)
                    && start.parse::<u16>().unwrap_or_default()
                        <= end.parse::<u16>().unwrap_or_default()
            }
            None => valid_port_number(segment),
        })
}

fn scan_args(request: &ScanRequest) -> Result<Vec<String>, String> {
    if !is_valid_port_spec(&request.ports) {
        return Err("La liste de ports doit contenir des ports ou plages valides, par exemple 22,80,443 ou 8000-8100.".to_string());
    }
    if request.profile == "discovery" && (request.version_detection || request.os_detection) {
        return Err(
            "La découverte d’hôtes ne peut pas inclure la détection de versions ou d’OS."
                .to_string(),
        );
    }

    let mut args = profile_args(&request.profile)?
        .iter()
        .map(|arg| (*arg).to_string())
        .collect::<Vec<_>>();
    args.insert(
        0,
        match request.timing.as_str() {
            "t3" => "-T3".to_string(),
            "t4" => "-T4".to_string(),
            _ => return Err("Niveau de timing inconnu.".to_string()),
        },
    );
    if request.skip_host_discovery && request.profile != "discovery" {
        args.push("-Pn".to_string());
    }
    match request.scan_type.as_str() {
        "auto" => {}
        "connect" => args.push("-sT".to_string()),
        "syn" => {
            if unsafe { libc::geteuid() } != 0 {
                return Err("Le scan SYN requiert des privilèges élevés. Choisis « Automatique » ou « Connexion TCP ».".to_string());
            }
            args.push("-sS".to_string());
        }
        _ => return Err("Méthode de scan inconnue.".to_string()),
    }
    if request.version_detection && request.profile != "discovery" {
        args.push("-sV".to_string());
    }
    if request.os_detection {
        args.push("-O".to_string());
    }
    if request.default_scripts {
        args.push("-sC".to_string());
    }
    if request.profile != "discovery" {
        if request.ports.trim().is_empty() {
            match request.profile.as_str() {
                "quick" => args.push("-F".to_string()),
                "web" => {
                    args.push("-p".to_string());
                    args.push("80,443,8080,8443".to_string());
                }
                "full_tcp" => args.push("-p-".to_string()),
                _ => {}
            }
        } else {
            args.push("-p".to_string());
            args.push(request.ports.trim().to_string());
        }
    }
    Ok(args)
}

fn list_exegol_containers() -> Result<ExegolContainerList, String> {
    let output = Command::new("docker")
        .args(["ps", "--format", "{{json .}}"])
        .output()
        .map_err(|error| format!("Impossible de lancer Docker : {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("Docker est indisponible pour Griffon : {stderr}"));
    }

    let containers = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|container| {
            let image = container["Image"].as_str()?.to_string();
            let labels = container["Labels"].as_str().unwrap_or_default();
            if !image.to_ascii_lowercase().contains("exegol")
                && !labels.to_ascii_lowercase().contains("exegol")
            {
                return None;
            }
            Some(ExegolContainer {
                name: container["Names"].as_str().unwrap_or_default().to_string(),
                image,
                status: container["Status"].as_str().unwrap_or_default().to_string(),
            })
        })
        .collect::<Vec<_>>();
    let message = if containers.is_empty() {
        "Aucun conteneur Exegol actif n’a été trouvé.".to_string()
    } else {
        format!("{} conteneur(s) Exegol disponible(s).", containers.len())
    };
    Ok(ExegolContainerList {
        ok: true,
        message,
        containers,
    })
}

fn nmap_command(request: &ScanRequest, args: &[String], target: &str) -> Result<Command, String> {
    match request.executor.as_str() {
        "local" => {
            let mut command = Command::new("nmap");
            command.args(args).args(["-oX", "-", "--", target]);
            Ok(command)
        }
        "exegol" => {
            let container = request.exegol_container.trim();
            if !is_valid_container_name(container) {
                return Err(
                    "Choisis un conteneur Exegol valide avant de lancer le scan.".to_string(),
                );
            }
            let mut command = Command::new("docker");
            command
                .args(["exec", container, "nmap"])
                .args(args)
                .args(["-oX", "-", "--", target]);
            Ok(command)
        }
        _ => Err("Exécuteur inconnu.".to_string()),
    }
}

fn displayed_command(request: &ScanRequest, args: &[String], target: &str) -> String {
    let prefix = if request.executor == "exegol" {
        format!("docker exec {} nmap", request.exegol_container.trim())
    } else {
        "nmap".to_string()
    };
    format!("{prefix} {} {target}", args.join(" "))
}

fn http_url(target: &str, port: &str, protocol: &str, path: &str) -> String {
    let host = match target.parse::<IpAddr>() {
        Ok(IpAddr::V6(_)) => format!("[{target}]"),
        _ => target.to_string(),
    };
    format!("{protocol}://{host}:{port}{path}")
}

fn page_title(body: &str) -> String {
    let lower = body.to_ascii_lowercase();
    let Some(start) = lower.find("<title") else {
        return "No HTML title found.".to_string();
    };
    let Some(content_start) = lower[start..].find('>').map(|offset| start + offset + 1) else {
        return "No HTML title found.".to_string();
    };
    let Some(end) = lower[content_start..]
        .find("</title>")
        .map(|offset| content_start + offset)
    else {
        return "No HTML title found.".to_string();
    };
    body[content_start..end].trim().to_string()
}

fn http_probe(request: HttpProbeRequest) -> Result<HttpProbeResponse, String> {
    let target = request.target.trim();
    if !is_valid_target(target) || !valid_port_number(&request.port) {
        return Err("La cible ou le port HTTP est invalide.".to_string());
    }
    if !matches!(request.protocol.as_str(), "http" | "https") {
        return Err("Seuls les services HTTP et HTTPS peuvent utiliser cette action.".to_string());
    }
    let (path, args) = match request.probe.as_str() {
        "headers" => ("", vec!["-k", "--max-time", "15", "-I", "-sS"]),
        "title" => ("", vec!["-k", "--max-time", "15", "-sS"]),
        "robots" => ("/robots.txt", vec!["-k", "--max-time", "15", "-sS"]),
        _ => return Err("Action HTTP inconnue.".to_string()),
    };
    let url = http_url(target, &request.port, &request.protocol, path);
    let mut command = match request.executor.as_str() {
        "local" => Command::new("curl"),
        "exegol" => {
            if !is_valid_container_name(request.exegol_container.trim()) {
                return Err(
                    "Choisis un conteneur Exegol valide avant de lancer cette action.".to_string(),
                );
            }
            let mut command = Command::new("docker");
            command.args(["exec", request.exegol_container.trim(), "curl"]);
            command
        }
        _ => return Err("Exécuteur inconnu.".to_string()),
    };
    command.args(&args).arg(&url);
    let command_display = if request.executor == "exegol" {
        format!(
            "docker exec {} curl {} {url}",
            request.exegol_container.trim(),
            args.join(" ")
        )
    } else {
        format!("curl {} {url}", args.join(" "))
    };
    let output = command
        .output()
        .map_err(|error| format!("Impossible de lancer curl : {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "curl a échoué : {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let raw_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let output = if request.probe == "title" {
        page_title(&raw_output)
    } else {
        raw_output
    };
    Ok(HttpProbeResponse {
        ok: true,
        target: target.to_string(),
        port: request.port,
        probe: request.probe,
        command: command_display,
        output,
        message: "Action terminée.".to_string(),
    })
}

fn attr(node: roxmltree::Node<'_, '_>, name: &str) -> String {
    node.attribute(name).unwrap_or_default().to_string()
}

fn parse_nmap_xml(xml: &str) -> Result<Vec<Host>, String> {
    // Nmap includes a DOCTYPE declaration in its normal XML output. roxmltree
    // rejects DTDs by default, so explicitly permit parsing it here. The parser
    // does not fetch the external Nmap DTD; it only reads the local XML output.
    let options = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..Default::default()
    };
    let document = roxmltree::Document::parse_with_options(xml, options)
        .map_err(|error| format!("Résultat Nmap XML invalide : {error}"))?;

    Ok(document
        .descendants()
        .filter(|node| node.has_tag_name("host"))
        .map(|host| {
            let address = host
                .children()
                .find(|node| {
                    node.has_tag_name("address") && node.attribute("addrtype") == Some("ipv4")
                })
                .or_else(|| host.children().find(|node| node.has_tag_name("address")))
                .map(|node| attr(node, "addr"))
                .unwrap_or_default();
            let hostname = host
                .descendants()
                .find(|node| node.has_tag_name("hostname"))
                .map(|node| attr(node, "name"))
                .unwrap_or_default();
            let state = host
                .children()
                .find(|node| node.has_tag_name("status"))
                .map(|node| attr(node, "state"))
                .unwrap_or_else(|| "unknown".to_string());
            let ports = host
                .descendants()
                .filter(|node| node.has_tag_name("port"))
                .filter_map(|port| {
                    let state = port
                        .children()
                        .find(|node| node.has_tag_name("state"))
                        .map(|node| attr(node, "state"))?;
                    if state != "open" {
                        return None;
                    }
                    let service = port.children().find(|node| node.has_tag_name("service"));
                    Some(Port {
                        port: attr(port, "portid"),
                        protocol: attr(port, "protocol"),
                        state,
                        service: service.map(|node| attr(node, "name")).unwrap_or_default(),
                        product: service
                            .map(|node| attr(node, "product"))
                            .unwrap_or_default(),
                        version: service
                            .map(|node| attr(node, "version"))
                            .unwrap_or_default(),
                        extra_info: service
                            .map(|node| attr(node, "extrainfo"))
                            .unwrap_or_default(),
                    })
                })
                .collect();
            Host {
                address,
                hostname,
                state,
                ports,
            }
        })
        .collect())
}

fn scan(request: ScanRequest) -> Result<ScanResponse, String> {
    let target = request.target.trim();
    if !is_valid_target(target) {
        return Err(
            "La cible doit être une adresse IP ou un nom d’hôte valide, sans plage ni liste."
                .to_string(),
        );
    }
    let args = scan_args(&request)?;
    let command = displayed_command(&request, &args, target);
    let started = Instant::now();
    let output = nmap_command(&request, &args, target)?
        .output()
        .map_err(|error| {
            format!("Impossible de lancer Nmap. Vérifie qu’il est installé : {error}")
        })?;
    let duration_ms = started.elapsed().as_millis();
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Nmap a échoué.".to_string()
        } else {
            format!("Nmap a échoué : {stderr}")
        });
    }
    let xml = String::from_utf8(output.stdout)
        .map_err(|_| "Nmap a renvoyé une sortie qui n’est pas du texte UTF-8.".to_string())?;
    let hosts = parse_nmap_xml(&xml)?;
    let response = ScanResponse {
        ok: true,
        message: "Analyse terminée.".to_string(),
        target: target.to_string(),
        profile: request.profile.clone(),
        command,
        duration_ms,
        hosts,
    };
    store_scan(&response, &request.executor)?;
    Ok(response)
}

fn split_message(raw: &str) -> (&str, &str) {
    let raw = raw.trim();
    let (head, payload) = raw.split_once(char::is_whitespace).unwrap_or((raw, ""));
    (head.strip_prefix("fn:").unwrap_or(head), payload.trim())
}

#[sabi_extern_fn]
pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
    let mut info = RVec::new();
    info.push(Tuple2(RString::from("author"), RString::from("Griffon")));
    info.push(Tuple2(RString::from("name"), RString::from("Nmap")));
    info.push(Tuple2(
        RString::from("description"),
        RString::from("Analyse réseau Nmap avec résultats structurés."),
    ));
    info.push(Tuple2(
        RString::from("UUID"),
        RString::from("5d059e05-c83f-4c59-9a85-13aa3fc1d721"),
    ));
    info.push(Tuple2(
        RString::from("function"),
        RString::from("scan/list_exegol_containers/http_probe/list_history/delete_history_entry/clear_history"),
    ));
    RResult::ROk(info)
}

#[sabi_extern_fn]
extern "C" fn handle_message(message: RString) -> RString {
    let (function, payload) = split_message(message.as_str());
    match function {
        "scan" => {
            let request = match serde_json::from_str::<ScanRequest>(payload) {
                Ok(request) => request,
                Err(error) => return json_error(format!("Paramètres de scan invalides : {error}")),
            };
            match scan(request) {
                Ok(response) => RString::from(serde_json::to_string(&response).unwrap_or_else(|error| {
                    serde_json::json!({ "ok": false, "message": format!("Erreur de sérialisation : {error}"), "hosts": [] }).to_string()
                })),
                Err(error) => json_error(error),
            }
        }
        "list_exegol_containers" => match list_exegol_containers() {
            Ok(response) => RString::from(serde_json::to_string(&response).unwrap_or_else(|error| {
                serde_json::json!({ "ok": false, "message": format!("Erreur de sérialisation : {error}"), "containers": [] }).to_string()
            })),
            Err(error) => RString::from(serde_json::json!({ "ok": false, "message": error, "containers": [] }).to_string()),
        },
        "http_probe" => match serde_json::from_str::<HttpProbeRequest>(payload) {
            Ok(request) => match http_probe(request) {
                Ok(response) => RString::from(serde_json::to_string(&response).unwrap_or_else(|error| serde_json::json!({ "ok": false, "message": format!("Erreur de sérialisation : {error}"), "output": "" }).to_string())),
                Err(error) => RString::from(serde_json::json!({ "ok": false, "message": error, "output": "" }).to_string()),
            },
            Err(error) => RString::from(serde_json::json!({ "ok": false, "message": format!("Paramètres HTTP invalides : {error}"), "output": "" }).to_string()),
        },
        "list_history" => match history_response() {
            Ok(response) => RString::from(serde_json::to_string(&response).unwrap_or_else(|error| serde_json::json!({ "ok": false, "message": format!("Erreur de sérialisation : {error}"), "scans": [] }).to_string())),
            Err(error) => RString::from(serde_json::json!({ "ok": false, "message": error, "scans": [] }).to_string()),
        },
        "delete_history_entry" => match serde_json::from_str::<DeleteHistoryRequest>(payload) {
            Ok(request) => match delete_history_entry(&request.id) {
                Ok(response) => RString::from(serde_json::to_string(&response).unwrap_or_else(|error| serde_json::json!({ "ok": false, "message": format!("Erreur de sérialisation : {error}"), "scans": [] }).to_string())),
                Err(error) => RString::from(serde_json::json!({ "ok": false, "message": error, "scans": [] }).to_string()),
            },
            Err(error) => RString::from(serde_json::json!({ "ok": false, "message": format!("Paramètres de suppression invalides : {error}"), "scans": [] }).to_string()),
        },
        "clear_history" => match clear_history() {
            Ok(response) => RString::from(serde_json::to_string(&response).unwrap_or_else(|error| serde_json::json!({ "ok": false, "message": format!("Erreur de sérialisation : {error}"), "scans": [] }).to_string())),
            Err(error) => RString::from(serde_json::json!({ "ok": false, "message": error, "scans": [] }).to_string()),
        },
        _ => json_error(format!("Fonction inconnue : {function}")),
    }
}

#[export_root_module]
pub fn get_library() -> PluginRoot_Ref {
    PluginRoot {
        plugin: PluginI {
            init,
            handle_message,
        }
        .leak_into_prefix(),
    }
    .leak_into_prefix()
}

#[cfg(test)]
mod tests {
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
}
