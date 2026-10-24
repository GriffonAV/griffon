pub struct PluginError {
    pub code: i32,
    pub message: String,
}

impl PluginError {
    pub fn new(code: i32, message: String) -> Self {
        Self { code, message }
    }
    pub fn method_not_found(message: String) -> Self {
        Self {
            code: -32601,
            message,
        }
    }
}

impl From<String> for PluginError {
    fn from(message: String) -> Self {
        Self { code: 1, message }
    }
}

impl From<&str> for PluginError {
    fn from(message: &str) -> Self {
        Self {
            code: 1,
            message: message.to_string(),
        }
    }
}
