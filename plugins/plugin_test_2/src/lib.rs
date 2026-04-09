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

    info.push(Tuple2(
        RString::from("author"),
        RString::from("Test Author2"),
    ));
    info.push(Tuple2(RString::from("name"), RString::from("Test Name2")));
    info.push(Tuple2(
        RString::from("description"),
        RString::from("Test Description2"),
    ));
    info.push(Tuple2(
        RString::from("UUID"),
        RString::from("6e9e800a-0d0c-4f74-8265-7b9ab0234582"),
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
