use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
enum ServerApiStatus {
    Success,
    PartialSuccess,
    Failed,
}

#[derive(Deserialize, Serialize)]
struct ServerMessage {
    api: String,
    status: ServerApiStatus,
    error: Option<Vec<(String, String)>>,
    library: Option<Uuid>,
    media: Option<u64>,
    format: Option<&'static str>,
    result: Option<String>,
    data: HashMap<String, String>,
}

impl Default for ServerMessage {
    fn default() -> Self {
        Self {
            api: "".into(),
            status: ServerApiStatus::Success,
            error: None,
            library: None,
            media: None,
            format: None,
            result: None,
            data: HashMap::new(),
        }
    }
}

impl ServerMessage {
    pub fn from_single_error<S1: Into<String>, S2: Into<String>>(
        at: S1,
        detail: S2,
        library: Option<Uuid>,
        media: Option<u64>,
    ) -> Self {
        Self {
            status: ServerApiStatus::Failed,
            error: Some(vec![(at.into(), detail.into())]),
            library,
            media,
            ..Self::default()
        }
    }

    pub fn to_json_string(&self, is_pretty: bool) -> String {
        let possible_result = if is_pretty {
            serde_json::to_string_pretty(&self)
        } else {
            serde_json::to_string(&self)
        };
        match possible_result {
            Ok(v) => v,
            Err(e) => format!(r#"{{"api": "server", "status": "Failed", "error": [["server side cannot serialize message json": "{}"]]}}"#, e).to_string()
        }
    }

    pub fn with_api_name(self, api_name: &str) -> Self {
        Self {
            api: api_name.into(),
            ..self
        }
    }
}
