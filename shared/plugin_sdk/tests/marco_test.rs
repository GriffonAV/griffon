use plugin_sdk::{PluginRouter, export_plugin, params::NoParams};

fn sample_handler(_: NoParams) -> Result<String, String> {
    Ok("ok".to_string())
}

fn build_router() -> PluginRouter {
    PluginRouter::new()
        .method("a", sample_handler)
        .method("b", sample_handler)
        .method("c", sample_handler)
}

// Expand the macro
export_plugin! {
    meta: PluginMeta {
        name: "TestPlugin",
        uuid: "1234",
        author: "Tester",
        description: "A test",
    },
    router: build_router,
}

#[test]
fn test_init_metadata_contains_functions() {
    // Call the generated init() function
    let result = init();

    // Unpack the RResult from abi_stable
    let Ok(info_vec) = result.into_result() else {
        panic!("init failed");
    };

    // Look for the "function" key in the returned Tuple2 array
    let function_tuple = info_vec
        .iter()
        .find(|tuple| tuple.0.as_str() == "function")
        .expect("function key missing from init metadata");

    // [x] The init metadata contains function = "a/b/c"
    assert_eq!(function_tuple.1.as_str(), "a/b/c");
}
