pub mod scanner_engine;

use abi_stable::std_types::{RVec, Tuple2};
use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RResult, RString},
};
use plugin_interface::{PluginI, PluginRoot, PluginRoot_Ref};

#[sabi_extern_fn]
pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
    let mut info = RVec::new();

    info.push(Tuple2(RString::from("author"), RString::from("DiaboloAB")));
    info.push(Tuple2(RString::from("name"), RString::from("Griffon Scan")));
    info.push(Tuple2(
        RString::from("description"),
        RString::from("Griffon Scan default plugin"),
    ));
    info.push(Tuple2(
        RString::from("UUID"),
        RString::from("123e4567-e89b-12d3-a456-426614174000"),
    ));
    info.push(Tuple2(RString::from("function"), RString::from("ping")));

    RResult::ROk(info)
}

#[sabi_extern_fn]
extern "C" fn handle_message(msg: RString) -> RString {
    print!("[LIB2](msg) Received message: {}", msg.as_str());

    match msg.as_str() {
        "fn:ping" => ping(),
        _ => RString::from(format!("ACK LIB2 {}\n", msg.as_str())),
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

fn ping() -> RString {
    RString::from("pong\n")
}
