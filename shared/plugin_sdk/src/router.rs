use abi_stable::std_types::RString;

type Handler = Box<dyn Fn(serde_json::Value) -> RString + Send + Sync>;

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

// Tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::NoParams;
    use serde::Deserialize;
    use serde::Serialize;

    #[derive(Deserialize)]
    struct MathArgs {
        val: i32,
    }

    #[derive(Serialize)]
    struct MathResult {
        result: i32,
    }

    fn double(args: MathArgs) -> Result<MathResult, String> {
        Ok(MathResult {
            result: args.val * 2,
        })
    }

    fn fail_always(_args: NoParams) -> Result<MathResult, String> {
        Err("Ceci est une erreur métier".to_string())
    }

    #[test]
    fn test_router_success() {
        let mut router = PluginRouter::new();
        router.add_method("double", double);

        let payload = r#"double {"val": 21}"#;
        let response = router.handle_raw(payload);

        assert_eq!(response.as_str(), r#"{"result":42}"#);
    }

    #[test]
    fn test_router_method_not_found() {
        let router = PluginRouter::new();

        let response = router.handle_raw(r#"unknown_method {}"#);

        assert!(response.as_str().contains("-32601"));
        assert!(response.as_str().contains("method not found"));
        assert!(response.as_str().contains("\"ok\":false"));
    }

    #[test]
    fn test_router_invalid_params() {
        let mut router = PluginRouter::new();
        router.add_method("double", double);

        let payload = r#"double {"val": "chaîne au lieu d'un i32"}"#;
        let response = router.handle_raw(payload);

        assert!(response.as_str().contains("-32602"));
        assert!(response.as_str().contains("invalid params"));
    }

    #[test]
    fn test_router_business_error() {
        let mut router = PluginRouter::new();
        router.add_method("fail", fail_always);

        let response = router.handle_raw(r#"fail {}"#);

        assert!(response.as_str().contains("\"code\":1"));
        assert!(response.as_str().contains("Ceci est une erreur métier"));
    }
}
