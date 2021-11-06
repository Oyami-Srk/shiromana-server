use serde::{Deserialize, Serialize};
use shiromana_rs::misc::Uuid;
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub enum ServerApiStatus {
    Success,
    PartialSuccess,
    Failed,
}

#[derive(Deserialize, Serialize)]
pub struct ServerMessage {
    api: String,
    status: ServerApiStatus,
    error: Option<Vec<(String, String)>>,
    library: Option<Uuid>,
    media: Option<u64>,
    format: Option<&'static str>,
    result: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    data: HashMap<String, String>,
    #[serde(skip_serializing)]
    is_preety: bool,
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

    pub fn to_json_string(&self) -> String {
        let possible_result = if self.is_preety {
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

impl Into<String> for ServerMessage {
    fn into(self) -> String {
        self.to_json_string()
    }
}