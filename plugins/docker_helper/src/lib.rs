use std::process::Command;

use std::collections::HashMap;
use std::sync::OnceLock;

use abi_stable::std_types::{RVec, Tuple2};
use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RResult, RString},
};
use plugin_interface::{PluginI, PluginRoot, PluginRoot_Ref};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Default)]
struct NoArgs {}

type Handler = Box<dyn Fn(serde_json::Value) -> RString + Send + Sync>;

fn command<I, O, F>(f: F) -> Handler
where
    I: serde::de::DeserializeOwned,
    O: Serialize,
    F: Fn(I) -> Result<O, String> + Send + Sync + 'static,
{
    Box::new(move |payload: serde_json::Value| -> RString {
        let input: I = match serde_json::from_value(payload) {
            Ok(v) => v,
            Err(e) => {
                let err_json = serde_json::json!({ "message": format!("invalid arguments: {e}") });
                return RString::from(err_json.to_string());
            }
        };

        match f(input) {
            Ok(output) => match serde_json::to_string(&output) {
                Ok(json) => RString::from(json),
                Err(e) => {
                    let err_json = serde_json::json!({ "message": format!("json serialize: {e}") });
                    RString::from(err_json.to_string())
                }
            },
            Err(e) => {
                let err_json = serde_json::json!({ "message": e });
                RString::from(err_json.to_string())
            }
        }
    })
}

#[derive(Serialize)]
struct Container {
    id: String,
    name: String,
    image: String,
    state: String,
    status: String,
}

#[derive(Serialize)]
struct ContainerList {
    containers: Vec<Container>,
}

fn registry() -> &'static HashMap<&'static str, Handler> {
    static REGISTRY: OnceLock<HashMap<&'static str, Handler>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut m: HashMap<&'static str, Handler> = HashMap::new();

        m.insert(
            "get_containers",
            command(|_: NoArgs| -> Result<ContainerList, String> {
                let output = Command::new("docker")
                    .arg("ps")
                    .arg("-a") // Include stopped containers
                    .arg("--format")
                    .arg("{{json .}}")
                    .output()
                    .map_err(|e| format!("Failed to execute docker command: {}", e))?;

                if !output.status.success() {
                    let err = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Docker CLI error: {}", err));
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut containers = Vec::new();

                for line in stdout.lines() {
                    if line.trim().is_empty() {
                        continue;
                    }

                    let v: serde_json::Value = match serde_json::from_str(line) {
                        Ok(val) => val,
                        Err(_) => continue, // Skip lines that fail to parse
                    };

                    containers.push(Container {
                        // Docker's JSON keys are capitalized by default (e.g., "ID", "Names")
                        id: v["ID"].as_str().unwrap_or("Unknown").to_string(),
                        name: v["Names"].as_str().unwrap_or("Unknown").to_string(),
                        image: v["Image"].as_str().unwrap_or("Unknown").to_string(),
                        state: v["State"].as_str().unwrap_or("Unknown").to_string(),
                        status: v["Status"].as_str().unwrap_or("Unknown").to_string(),
                    });
                }

                Ok(ContainerList { containers })
            }),
        );

        m
    })
}

fn parse_payload(raw_payload: &str) -> Result<serde_json::Value, String> {
    if raw_payload.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(raw_payload).map_err(|e| format!("invalid payload json: {e}"))
}

#[sabi_extern_fn]
pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
    let mut info = RVec::new();

    info.push(Tuple2(RString::from("author"), RString::from("DiaboloAB")));
    info.push(Tuple2(
        RString::from("name"),
        RString::from("Docker Helper"),
    ));
    info.push(Tuple2(
        RString::from("description"),
        RString::from("A simple plugin that display your docker container."),
    ));
    info.push(Tuple2(
        RString::from("UUID"),
        RString::from("b86fcda4-4e8d-4988-b524-8bbd9ce09abc"),
    ));
    info.push(Tuple2(
        RString::from("function"),
        RString::from("get_containers"),
    ));

    RResult::ROk(info)
}

fn split_message(raw: &str) -> (&str, &str) {
    let raw = raw.trim();
    let (head, rest) = match raw.split_once(char::is_whitespace) {
        Some((h, r)) => (h, r.trim()),
        None => (raw, ""),
    };
    let name = head.strip_prefix("fn:").unwrap_or(head);
    (name, rest)
}

#[sabi_extern_fn]
extern "C" fn handle_message(msg: RString) -> RString {
    let raw = msg.as_str().trim();
    log::info!("[LIBDOCKERH] Received message: {}", raw);

    let (function, raw_payload) = split_message(raw);

    let payload = match parse_payload(raw_payload) {
        Ok(v) => v,
        Err(_e) => return RString::from(format!("ERR invalid payload json: {raw_payload}")),
        // Err(e) => return RString::from(format!("ERR {e}")),
    };

    match registry().get(function) {
        Some(handler) => handler(payload),
        None => RString::from(format!("ERR unknown function: {function}")),
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
