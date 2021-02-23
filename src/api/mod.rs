use std::process;
use std::str::FromStr;
use std::{io, path};

use actix_web::error::PayloadError::Http2Payload;
use actix_web::{HttpRequest, HttpResponse, Responder};
use qstring::QString;
use serde::{Deserialize, Serialize};
use shiromana_rs::library::Library;
use shiromana_rs::media::{Media, MediaType};
use shiromana_rs::misc::Uuid;
use tokio::sync::mpsc::Sender;

use super::AppState;

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
    library_uuid: Option<Uuid>,
    media_id: Option<u64>,
    format: Option<&'static str>,
    value: Option<String>,
}

impl Default for ServerMessage {
    fn default() -> Self {
        Self {
            api: "".into(),
            status: ServerApiStatus::Success,
            error: None,
            library_uuid: None,
            media_id: None,
            format: None,
            value: None,
        }
    }
}

mod error_msg {
    pub const FOLDER_NOT_EXISTS: &str =
        "Field path: `{}` is not exists on the disk of server. Or it is not a folder.";
    pub const FILE_NOT_EXISTS: &str =
        "Field path: `{}` is not exists on the disk of server. Or it is not a file.";
    pub const LIBRARY_NOT_OPENED: &str = "Library `{}` is not opened.";
}

macro_rules! simple_format {
    ($s:expr, $( $e:expr ),*) => {{
        let mut s: String = $s.into();
        $(
            let e: String = $e.into();
            s = s.replacen("{}", &e, 1);
        )*
        s
    }};
}

impl ServerMessage {
    pub fn from_single_error<S1: Into<String>, S2: Into<String>>(
        at: S1,
        detail: S2,
        library_uuid: Option<Uuid>,
        media_id: Option<u64>,
    ) -> Self {
        Self {
            status: ServerApiStatus::Failed,
            error: Some(vec![(at.into(), detail.into())]),
            library_uuid,
            media_id,
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

pub async fn dispatcher(api_name: &str, context: &AppState, req: HttpRequest) -> impl Responder {
    let qs = QString::from(req.query_string());

    // most used value, for simplify code
    let is_pretty = qs.has("json_pretty") && qs.get("json_pretty").unwrap() != "false";

    let library_uuid = match qs.get("uuid") {
        Some(s) => match s.parse::<Uuid>() {
            Ok(v) => Some(v),
            Err(e) => {
                return HttpResponse::BadRequest().body(
                    ServerMessage::from_single_error(
                        "parameter",
                        format!("Failed to parse parameter `uuid` into `Uuid`: {}.", e),
                        None,
                        None,
                    )
                    .with_api_name(api_name)
                    .to_json_string(is_pretty),
                );
            }
        },
        None => None,
    };

    let media_id = match qs.get("id") {
        Some(s) => match s.parse::<u64>() {
            Ok(v) => Some(v),
            Err(e) => {
                return HttpResponse::BadRequest().body(
                    ServerMessage::from_single_error(
                        "parameter",
                        format!("Failed to parse parameter `id` into `u64`: {}.", e),
                        None,
                        None,
                    )
                    .with_api_name(api_name)
                    .to_json_string(is_pretty),
                );
            }
        },
        None => None,
    };

    // Some useful function and macros
    let qs_get_string = |k: &str| match qs.get(k) {
        Some(v) => Some(v.to_string()),
        None => None,
    };

    macro_rules! qs_get_or_bad {
        ($key:expr) => {{
            match qs.get($key) {
                Some(v) => v,
                None => {
                    return ServerMessage::from_single_error(
                        "parameter",
                        format!("parameter `{}` does not exists.", $key),
                        library_uuid,
                        media_id,
                    )
                }
            }
        }};
    };

    macro_rules! qs_get_and_parse {
        ($key:expr, $t: ty) => {{
            match qs_get_or_bad!($key).parse::<$t>() {
                Ok(v) => v,
                Err(e) => {
                    return ServerMessage::from_single_error(
                        "parameter",
                        format!(
                            "Failed to parse parameter `{}` into `{}`: {}.",
                            $key,
                            std::any::type_name::<$t>(),
                            e
                        ),
                        library_uuid,
                        media_id,
                    )
                }
            }
        }};
    }

    macro_rules! extract_option_of_param {
        ($key: expr, $opt: expr) => {{
            match $opt {
                Some(v) => v,
                None => {
                    return ServerMessage::from_single_error(
                        "parameter",
                        format!("parameter `{}` does not exists.", $key),
                        library_uuid,
                        media_id,
                    )
                }
            }
        }};
    }

    let wrapped_from_single_error =
        |at, detail| ServerMessage::from_single_error(at, detail, library_uuid, media_id);

    // logic matches
    let server_msg = async || {
        match api_name {
            "status" => ServerMessage::default(),
            "open_library" => {
                let path = qs_get_or_bad!("path");
                println!("Opening library located at `{}`", path);
                if !path::PathBuf::from(path).is_dir() {
                    return wrapped_from_single_error(
                        "library",
                        simple_format!(error_msg::FOLDER_NOT_EXISTS, path),
                    );
                }
                let lib = match Library::open(path.to_string()) {
                    Ok(v) => v,
                    Err(e) => {
                        return wrapped_from_single_error(
                            "library",
                            format!("Failed to open library at `{}`: {}.", path, e),
                        );
                    }
                };
                let library_uuid = lib.uuid.clone();

                {
                    let mut opened_library = context.opened_libraries.lock().await;
                    let uuid = lib.uuid.clone();
                    opened_library.insert(uuid, lib);
                }
                ServerMessage {
                    library_uuid: Some(library_uuid),
                    ..ServerMessage::default()
                }
            }
            "close_library" => {
                {
                    let mut opened_library = context.opened_libraries.lock().await;
                    let library_uuid = extract_option_of_param!("uuid", library_uuid);

                    match opened_library.remove(&library_uuid) {
                        Some(v) => drop(v),
                        None => {
                            return wrapped_from_single_error(
                                "library",
                                format!("Failed to close library, maybe not opened at all.",),
                            );
                        }
                    }
                };
                ServerMessage::default()
            }
            "create_library" => {
                let path = qs_get_or_bad!("path");
                println!("Creating library located at `{}`", path);
                if !path::PathBuf::from(path).is_dir() {
                    return wrapped_from_single_error(
                        "library",
                        simple_format!(error_msg::FOLDER_NOT_EXISTS, path),
                    );
                }
                let lib = Library::create(
                    path.to_string(),
                    qs_get_string("library_name").unwrap(),
                    qs_get_string("master_name"),
                    qs_get_string("media_folder"),
                );
                let lib = match lib {
                    Ok(v) => v,
                    Err(e) => {
                        return wrapped_from_single_error(
                            "library",
                            format!("Failed to create library: {}", e),
                        );
                    }
                };
                let uuid = lib.uuid.clone();
                {
                    let mut opened_library = context.opened_libraries.lock().await;
                    opened_library.insert(lib.uuid, lib);
                };
                ServerMessage {
                    library_uuid: Some(uuid),
                    ..ServerMessage::default()
                }
            }
            "add_media" => {
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let path = qs_get_or_bad!("path");
                println!("Trying to add media `{}`.", path);
                if !path::PathBuf::from(path).is_file() {
                    return wrapped_from_single_error(
                        "media",
                        simple_format!(error_msg::FILE_NOT_EXISTS, path),
                    );
                };
                let mut opened_libraries = context.opened_libraries.lock().await;
                let mut library = opened_libraries.get_mut(&uuid);
                if let None = library {
                    return wrapped_from_single_error(
                        "library",
                        simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                    );
                }
                let mut library = library.unwrap();
                let id = library.add_media(
                    path.to_string(),
                    MediaType::from_str(qs.get("kind").unwrap()).unwrap(),
                    qs_get_string("sub_kind"),
                    qs_get_string("kind_addition"),
                    qs_get_string("caption"),
                    qs_get_string("comment"),
                );
                drop(opened_libraries);
                let id = match id {
                    Ok(id) => id,
                    Err(e) => {
                        return wrapped_from_single_error(
                            "media",
                            format!("Failed to add media: {}", e),
                        );
                    }
                };
                if qs.has("delete") && qs.get("delete").unwrap() != "false" {
                    // remove original file
                    if let Err(e) = std::fs::remove_file(path) {
                        return ServerMessage {
                            status: ServerApiStatus::PartialSuccess,
                            error: Some(vec![(
                                "media".into(),
                                format!("Failed to remove original file: {}", e),
                            )]),
                            ..ServerMessage::default()
                        };
                    }
                }
                ServerMessage {
                    media_id: Some(id),
                    ..ServerMessage::default()
                }
            }
            "remove_media" => {
                let id = extract_option_of_param!("id", media_id);
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let mut opened_libraries = context.opened_libraries.lock().await;
                let mut library = opened_libraries.get_mut(&uuid);
                let mut library = match library {
                    Some(v) => v,
                    None => {
                        return wrapped_from_single_error(
                            "library",
                            simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                        );
                    }
                };
                library.remove_media(id);
                ServerMessage::default()
            }
            "get_media" => {
                let id = extract_option_of_param!("id", media_id);
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let opened_libraries = context.opened_libraries.lock().await;
                let library = opened_libraries.get(&uuid);
                let library = match library {
                    Some(v) => v,
                    None => {
                        return wrapped_from_single_error(
                            "library",
                            simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                        );
                    }
                };
                let media = library.get_media(id);
                match media {
                    Ok(v) => ServerMessage {
                        format: Some("json"),
                        value: Some(serde_json::to_string(&v).unwrap()),
                        ..ServerMessage::default()
                    },
                    Err(e) => {
                        wrapped_from_single_error("media", format!("Failed to get media: {}.", e))
                    }
                }
            }
            "update_media" => {
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let media = qs_get_or_bad!("media");
                let mut media: Media = match serde_json::from_str(media) {
                    Ok(v) => v,
                    Err(e) => {
                        return wrapped_from_single_error(
                            "parameter",
                            format!("Failed to parse parameter `media`: {}.", e),
                        );
                    }
                };
                let mut opened_libraries = context.opened_libraries.lock().await;
                let mut library = opened_libraries.get_mut(&uuid);
                let mut library = match library {
                    Some(v) => v,
                    None => {
                        return wrapped_from_single_error(
                            "library",
                            simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                        );
                    }
                };
                if let Err(e) = library.update_media(&mut media) {
                    return wrapped_from_single_error(
                        "media",
                        format!("Failed to update media: {}.", e),
                    );
                }
                ServerMessage::default()
            }
            "create_series" => {
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let mut opened_libraries = context.opened_libraries.lock().await;
                let mut library = opened_libraries.get_mut(&uuid);
                let mut library = match library {
                    Some(v) => v,
                    None => {
                        return wrapped_from_single_error(
                            "library",
                            simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                        );
                    }
                };
                match library.create_series(qs_get_string("caption"), qs_get_string("comment")) {
                    Ok(v) => ServerMessage {
                        library_uuid: Some(uuid),
                        format: Some("plain"),
                        value: Some(v.into()),
                        ..ServerMessage::default()
                    },
                    Err(e) => wrapped_from_single_error(
                        "series",
                        format!("Failed to create series: {}.", e),
                    ),
                }
            }
            "delete_series" => {
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let series_uuid = qs_get_and_parse!("series_uuid", Uuid);
                let mut opened_libraries = context.opened_libraries.lock().await;
                let mut library = opened_libraries.get_mut(&uuid);
                let mut library = match library {
                    Some(v) => v,
                    None => {
                        return wrapped_from_single_error(
                            "library",
                            simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                        );
                    }
                };
                match library.delete_series(&series_uuid) {
                    Ok(_) => ServerMessage::default(),
                    Err(e) => wrapped_from_single_error(
                        "series",
                        format!("Failed to delete series `{}`: {}.", series_uuid, e),
                    ),
                }
            }
            "add_to_series" => {
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let series_uuid = qs_get_and_parse!("series_uuid", Uuid);
                let id = extract_option_of_param!("id", media_id);
                let no = match qs.get("no") {
                    Some(v) => match v.parse::<u64>() {
                        Ok(v) => Some(v),
                        Err(e) => {
                            return wrapped_from_single_error(
                                "parameter",
                                format!("Failed to parse field `no` into `u64`: {}.", e),
                            );
                        }
                    },
                    None => None,
                };

                let mut opened_libraries = context.opened_libraries.lock().await;
                let mut library = opened_libraries.get_mut(&uuid);
                let mut library = match library {
                    Some(v) => v,
                    None => {
                        return wrapped_from_single_error(
                            "library",
                            simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                        );
                    }
                };

                match library.add_to_series(id, &series_uuid, no) {
                    Ok(_) => ServerMessage::default(),
                    Err(e) => wrapped_from_single_error(
                        "series",
                        format!("Failed to add media to series `{}`: {}.", series_uuid, e),
                    ),
                }
            }
            "remove_from_series" => {
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let id = extract_option_of_param!("id", media_id);

                let mut opened_libraries = context.opened_libraries.lock().await;
                let mut library = opened_libraries.get_mut(&uuid);
                let mut library = match library {
                    Some(v) => v,
                    None => {
                        return wrapped_from_single_error(
                            "library",
                            simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                        );
                    }
                };

                match library.remove_from_series(id) {
                    Ok(_) => ServerMessage::default(),
                    Err(e) => wrapped_from_single_error(
                        "series",
                        format!("Failed to remove media from series: {}.", e),
                    ),
                }
            }
            "update_series_no" => {
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let id = extract_option_of_param!("id", media_id);
                let no = qs_get_and_parse!("no", u64);
                let insert = qs.has("insert") && qs.get("insert").unwrap() != "false";

                let mut opened_libraries = context.opened_libraries.lock().await;
                let mut library = opened_libraries.get_mut(&uuid);
                let mut library = match library {
                    Some(v) => v,
                    None => {
                        return wrapped_from_single_error(
                            "library",
                            simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                        );
                    }
                };

                match library.update_series_no(id, no, insert) {
                    Ok(_) => ServerMessage::default(),
                    Err(e) => wrapped_from_single_error(
                        "series",
                        format!("Failed update media's no in series: {}.", e),
                    ),
                }
            }
            "trim_series_no" => {
                let uuid = extract_option_of_param!("uuid", library_uuid);
                let series_uuid = qs_get_and_parse!("series_uuid", Uuid);
                let mut opened_libraries = context.opened_libraries.lock().await;

                let mut library = opened_libraries.get_mut(&uuid);
                let mut library = match library {
                    Some(v) => v,
                    None => {
                        return wrapped_from_single_error(
                            "library",
                            simple_format!(error_msg::LIBRARY_NOT_OPENED, uuid),
                        );
                    }
                };

                match library.trim_series_no(&series_uuid) {
                    Ok(_) => ServerMessage::default(),
                    Err(e) => wrapped_from_single_error(
                        "series",
                        format!("Failed trim series no for series `{}`: {}.", series_uuid, e),
                    ),
                }
            }
            _ => ServerMessage {
                status: ServerApiStatus::Failed,
                error: Some(vec![("api".into(), "Not a valid api request.".into())]),
                ..ServerMessage::default()
            },
        }
    };
    let server_msg = server_msg().await;
    let server_msg = ServerMessage {
        api: api_name.into(),
        ..server_msg
    };
    let server_msg = if server_msg.library_uuid.is_none() && library_uuid.is_some() {
        ServerMessage {
            library_uuid,
            ..server_msg
        }
    } else {
        server_msg
    };

    let server_msg = if server_msg.media_id.is_none() && media_id.is_some() {
        ServerMessage {
            library_uuid,
            ..server_msg
        }
    } else {
        server_msg
    };
    let msg_string = server_msg.to_json_string(is_pretty);
    return match &server_msg.status {
        ServerApiStatus::Success => HttpResponse::Ok().body(msg_string),
        ServerApiStatus::PartialSuccess => HttpResponse::Ok().body(msg_string),
        ServerApiStatus::Failed => HttpResponse::BadRequest().body(msg_string),
    };
}
