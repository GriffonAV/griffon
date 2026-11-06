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

    pub fn method<I, O, E, F>(mut self, name: &'static str, f: F) -> Self
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
        self
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

    pub fn call_json(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, crate::error::PluginError> {
        let json_raw = params.to_string();

        let raw = format!("{method} {json_raw}");
        let response = self.handle_raw(&raw);
        let json_val: serde_json::Value = serde_json::from_str(response.as_str()).unwrap();
        if let Some(obj) = json_val.as_object()
            && obj.get("ok") == Some(&serde_json::Value::Bool(false))
        {
            let code = obj.get("code").and_then(|c| c.as_i64()).unwrap_or(-32603) as i32;
            let msg = obj
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown error");
            return Err(crate::error::PluginError::new(code, msg.to_string()));
        }
        Ok(json_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::NoParams;
    use serde::Deserialize;

    // --- Handlers ---
    fn echo(val: serde_json::Value) -> Result<serde_json::Value, String> {
        Ok(val)
    }

    fn ping(_args: NoParams) -> Result<String, String> {
        Ok("pong".to_string())
    }

    #[derive(Deserialize)]
    struct StrictArgs {
        target: String,
    }

    fn strict_method(args: StrictArgs) -> Result<String, String> {
        Ok(format!("target is {}", args.target))
    }

    fn fail_always(_args: NoParams) -> Result<String, String> {
        Err("boom".to_string())
    }

    fn panic_always(_args: NoParams) -> Result<String, String> {
        panic!("this is a deliberate panic");
    }

    // --- Router Builder ---
    fn build_test_router() -> PluginRouter {
        PluginRouter::new()
            .method("echo", echo)
            .method("ping", ping)
            .method("strict", strict_method)
            .method("fail", fail_always)
            .method("panic", panic_always)
    }

    // --- Tests ---
    #[test]
    fn test_echo_with_and_without_prefix() {
        let router = build_test_router();

        // [x] `fn:echo {"a":1}` returns `{"a":1}`
        let res1 = router.handle_raw(r#"fn:echo {"a":1}"#);
        assert_eq!(res1.as_str(), r#"{"a":1}"#);

        // [x] `echo {"a":1}` (no `fn:`) works too
        let res2 = router.handle_raw(r#"echo {"a":1}"#);
        assert_eq!(res2.as_str(), r#"{"a":1}"#);
    }

    #[test]
    fn test_no_params_variants() {
        let router = build_test_router();

        // [x] `fn:ping`, `fn:ping {}` and `fn:ping null` all reach a NoParams handler
        assert_eq!(router.handle_raw("fn:ping").as_str(), r#""pong""#);
        assert_eq!(router.handle_raw("fn:ping {}").as_str(), r#""pong""#);
        assert_eq!(router.handle_raw("fn:ping null").as_str(), r#""pong""#);
    }

    #[test]
    fn test_unknown_method() {
        let router = build_test_router();

        // [x] An unknown method returns code -32601
        let res = router.handle_raw("unknown_method {}");
        assert!(res.as_str().contains("-32601"));
        assert!(res.as_str().contains("method not found"));
    }

    #[test]
    fn test_invalid_params() {
        let router = build_test_router();

        // [x] A missing field returns code -32602
        let res_missing = router.handle_raw(r#"strict {}"#);
        assert!(res_missing.as_str().contains("-32602"));
        assert!(res_missing.as_str().contains("missing field `target`"));

        // [x] Non-JSON params return -32602
        let res_bad_json = router.handle_raw(r#"strict {bad_json}"#);
        assert!(res_bad_json.as_str().contains("-32602"));
    }

    #[test]
    fn test_business_error() {
        let router = build_test_router();

        // [x] A handler that returns Err("boom".to_string()) produces code 1 and message boom
        let res = router.handle_raw("fail");
        let json: serde_json::Value = serde_json::from_str(res.as_str()).unwrap();

        assert_eq!(json["ok"], false);
        assert_eq!(json["code"], 1);
        assert_eq!(json["message"], "boom");
    }

    #[test]
    fn test_panic_safety() {
        let router = build_test_router();

        // [x] A handler that panics produces -32603, and the test process keeps running
        let res = router.handle_raw("panic");
        let json: serde_json::Value = serde_json::from_str(res.as_str()).unwrap();

        assert_eq!(json["ok"], false);
        assert_eq!(json["code"], -32603);
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("this is a deliberate panic")
        );
    }

    #[test]
    fn test_method_registration_order() {
        let router = build_test_router();

        // [x] method_names() keeps registration order
        let names = router.method_names();
        assert_eq!(names, vec!["echo", "ping", "strict", "fail", "panic"]);
    }

    #[test]
    fn test_call_json() {
        let router = build_test_router();

        let params = serde_json::json!({ "a": 1 });
        let result = router
            .call_json("echo", params)
            .expect("Call should succeed");
        assert_eq!(result, serde_json::json!({ "a": 1 }));

        let err = router
            .call_json("unknown", serde_json::json!({}))
            .unwrap_err();
        assert_eq!(err.code, -32601);
    }
}
