use super::error::Result;
use paste::paste;
use serde::{Deserialize, Serialize};
use shiromana_rs::misc::Uuid;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone)]
pub enum ServerApiStatus {
    Success,
    PartialSuccess,
    Failed,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ServerMessage {
    pub api: String,
    pub status: ServerApiStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Vec<(String, String)>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub library: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub data: HashMap<String, String>,
    #[serde(skip_serializing)]
    pub is_preety: bool,
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
            is_preety: true,
        }
    }
}

macro_rules! generate_with_function {
    ($name: ident, $type: ty) => {
        paste! {
            pub fn [<with_ $name>]<F: Into<$type>>(self, v: F) -> Self {
                Self {
                    $name: v.into(),
                    ..self
                }
            }
        }
    };

    ($name: ident, $type: ty, $process: ident) => {
        paste! {
            pub fn [<with_ $name>]<F: Into<$type>>(self, v: F) -> Self {
                Self {
                    $name: $process(v.into()),
                    ..self
                }
            }
        }
    };
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

    pub fn with_single_error<S1: Into<String>, S2: Into<String>>(
        self,
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
            ..self
        }
    }

    pub fn to_json_string(&self) -> String {
        let possible_result = if self.is_preety {
            serde_json::to_string_pretty(&self)
        } else {
            serde_json::to_string(&self)
        };
        match possible_result {
            Ok(v) => v + "\n",
            Err(e) => format!(r#"{{"api": "server", "status": "Failed", "error": [["server side cannot serialize message json": "{}"]]}}\n"#, e).to_string()
        }
    }

    pub fn with_pretty_json(self) -> Self {
        Self {
            is_preety: true,
            ..self
        }
    }

    pub fn without_pretty_json(self) -> Self {
        Self {
            is_preety: false,
            ..self
        }
    }

    generate_with_function!(api, String);
    generate_with_function!(library, Uuid, Some);
    generate_with_function!(media, u64, Some);
    generate_with_function!(result, String, Some);
    generate_with_function!(format, &'static str, Some);
}

impl Into<String> for ServerMessage {
    fn into(self) -> String {
        self.to_json_string()
    }
}

/*
impl From<Result<ServerMessage>> for ServerMessage {
    fn from(result: Result<ServerMessage>) -> Self {
        match result {
            Ok(v) => v,
            Err(e) =>
        }
    }
} */
