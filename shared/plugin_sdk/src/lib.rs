use abi_stable::std_types::RString;
mod error;
pub mod params;
mod wire;

type Handler = Box<dyn Fn(serde_json::Value) -> RString + Send + Sync>;

pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

#[doc(hidden)]
pub mod __private {
    pub use abi_stable;
    pub use plugin_interface;
}

#[derive(Default)]
pub struct PluginRouter {
    methods: Vec<(&'static str, Handler)>,
}

impl PluginRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_method<I, O, E, F>(&mut self, name: &'static str, f: F)
    where
        I: serde::de::DeserializeOwned,
        O: serde::Serialize,
        E: Into<crate::error::PluginError>,
        F: Fn(I) -> Result<O, E> + Send + Sync + 'static,
    {
        let handler: Handler = Box::new(move |payload: serde_json::Value| -> RString {
            let input: I = match serde_json::from_value(payload) {
                Ok(v) => v,
                Err(e) => {
                    let err_json = serde_json::json!({
                        "ok": false,
                        "code": -32602,
                        "message": format!("invalid params: {e}")
                    });
                    return RString::from(err_json.to_string());
                }
            };

            match f(input) {
                Ok(output) => match serde_json::to_string(&output) {
                    Ok(json) => RString::from(json),
                    Err(e) => {
                        let err_json = serde_json::json!({
                            "ok": false,
                            "code": -32603,
                            "message": format!("json serialize: {e}")
                        });
                        RString::from(err_json.to_string())
                    }
                },
                Err(e) => {
                    let plugin_err: crate::error::PluginError = e.into();
                    let err_json = serde_json::json!({
                        "ok": false,
                        "code": plugin_err.code,
                        "message": plugin_err.message
                    });
                    RString::from(err_json.to_string())
                }
            }
        });

        self.methods.push((name, handler));
    }

    pub fn handle_raw(&self, raw: &str) -> ::abi_stable::std_types::RString {
        let (method_name, raw_payload) = crate::wire::split_message(raw);

        let handler = match self.methods.iter().find(|(name, _)| *name == method_name) {
            Some((_, h)) => h,
            None => {
                let err = serde_json::json!({
                    "ok": false,
                    "code": -32601,
                    "message": format!("method not found: {}", method_name)
                });
                return ::abi_stable::std_types::RString::from(err.to_string());
            }
        };

        let payload = match crate::wire::parse_payload(raw_payload.as_str()) {
            Ok(v) => v,
            Err(e) => {
                let err = serde_json::json!({
                    "ok": false,
                    "code": -32602,
                    "message": format!("invalid params: {}", e)
                });
                return ::abi_stable::std_types::RString::from(err.to_string());
            }
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(payload)));

        match result {
            Ok(rstring) => rstring,
            Err(err) => {
                let panic_msg = if let Some(s) = err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };

                let err_json = serde_json::json!({
                    "ok": false,
                    "code": -32603,
                    "message": format!("plugin panicked: {}", panic_msg)
                });
                ::abi_stable::std_types::RString::from(err_json.to_string())
            }
        }
    }

    pub fn method_names(&self) -> Vec<&str> {
        self.methods.iter().map(|(name, _)| *name).collect()
    }
}

pub struct PluginMeta {
    pub name: String,
    pub uuid: String,
    pub author: String,
    pub description: String,
}

#[macro_export]
macro_rules! export_plugin {
    (
        meta: PluginMeta {
            name: $name:expr,
            uuid: $uuid:expr,
            author: $author:expr,
            description: $desc:expr $(,)?,
        },
        router: $build_router:expr $(,)?

        // $( on_init: $on_init:expr $(,)? )?
    ) => {
        use $crate::__private::abi_stable::prefix_type::PrefixTypeTrait;
        use $crate::__private::abi_stable::std_types::{RResult, RString, RVec, Tuple2};
        use $crate::__private::plugin_interface::{PluginI, PluginRoot, PluginRoot_Ref};
        static ROUTER: std::sync::OnceLock<$crate::PluginRouter> = std::sync::OnceLock::new();

        #[::abi_stable::sabi_extern_fn]
        pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
            let router = ROUTER.get_or_init($build_router);
            let mut info = RVec::new();

            info.push(Tuple2(RString::from("name"), RString::from($name)));
            info.push(Tuple2(RString::from("UUID"), RString::from($uuid)));
            info.push(Tuple2(RString::from("description"), RString::from($desc)));
            info.push(Tuple2(RString::from("author"), RString::from($author)));
            info.push(Tuple2(
                RString::from("function"),
                RString::from(router.method_names().join("/")),
            ));
            info.push(Tuple2(
                RString::from("sdk_version"),
                RString::from($crate::SDK_VERSION),
            ));

            RResult::ROk(info)
        }

        #[::abi_stable::sabi_extern_fn]
        extern "C" fn handle_message(msg: RString) -> RString {
            let router = ROUTER.get_or_init($build_router);
            RString::from(router.handle_raw(msg.as_str()))
        }

        #[::abi_stable::export_root_module]
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
    };
}
