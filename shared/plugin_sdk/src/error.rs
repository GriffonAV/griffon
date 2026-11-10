use std::fmt;

pub struct PluginError {
    pub code: i32,
    pub message: String,
}

impl PluginError {
    pub fn new(code: i32, message: String) -> Self {
        Self { code, message }
    }
    pub fn method_not_found(msg: impl std::fmt::Display) -> Self {
        Self {
            code: -32601,
            message: format!("method not found: {}", msg),
        }
    }
    pub fn invalid_params(msg: impl std::fmt::Display) -> Self {
        Self {
            code: -32602,
            message: format!("invalid params: {}", msg),
        }
    }

    pub fn internal(msg: impl std::fmt::Display) -> Self {
        Self {
            code: -32603,
            message: format!("internal error: {}", msg),
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

impl fmt::Debug for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PluginError")
            .field("code", &self.code)
            .field("message", &self.message)
            .finish()
    }
}
