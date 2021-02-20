use std::{io, path};
use std::process;

use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::error::PayloadError::Http2Payload;
use qstring::QString;
use shiromana_rs::library::Library;
use shiromana_rs::media::{Media, MediaType};
use shiromana_rs::misc::Uuid;
use tokio::sync::mpsc::Sender;

use super::AppState;

pub async fn dispatcher(api_name: &str, context: &AppState, req: HttpRequest) -> impl Responder {
    let qs = QString::from(req.query_string());
    let qs_get_string = |k: &str| {
        match qs.get(k) {
            Some(v) => Some(v.to_string()),
            None => None
        }
    };
    macro_rules! qs_get_uuid {
        () => {
        qs
        };
    }
    match api_name {
        "test" => {
            HttpResponse::Ok().body("Shiromana-Server is running nicely, safe and sound.")
        }
        "open_library" => {
            let path = match qs.get("path") {
                Some(v) => v,
                None => return HttpResponse::BadRequest().body("open_library require a valid `path` field.")
            };
            println!("Opening library located at `{}`", path);
            if !path::PathBuf::from(path).is_dir() {
                return HttpResponse::BadRequest().body("open_library require `path` field being existed on the disk of server.");
            }
            let lib = match Library::open(path.to_string()) {
                Ok(v) => v,
                Err(e) => return HttpResponse::BadRequest().body(format!("open_library failed because of: {}", e))
            };
            let library_uuid = lib.uuid.clone();

            async {
                let mut opened_library = context.opened_libraries.lock().await;
                let uuid = lib.uuid.clone();
                opened_library.insert(uuid, lib);
            }.await;

            HttpResponse::Ok().body(format!("Opened library with Uuid: {}", library_uuid))
        }
        "close_library" => {
            let uuid = match qs.get("uuid") {
                Some(v) => v,
                None => return HttpResponse::BadRequest().body("close_library require a valid `uuid` field.")
            };
            let uuid = match Uuid::from_str(uuid) {
                Ok(v) => v,
                Err(e) => return HttpResponse::BadRequest().body("close_library require `uuid` field being a valid UUID string.")
            };
            {
                let mut opened_library = context.opened_libraries.lock().await;

                match opened_library.remove(&uuid) {
                    Some(v) => drop(v),
                    None => return HttpResponse::BadRequest().body("close_library failed to close this library")
                }
            }
            HttpResponse::Ok().body("Success")
        }
        "create_library" => {
            if qs.has("path") == false || qs.has("library_name") == false {
                return HttpResponse::BadRequest().body("create_library needs field `path` and `library_name`");
            }
            let path = qs.get("path").unwrap();
            println!("Creating library located at `{}`", path);
            if !path::PathBuf::from(path).is_dir() {
                return HttpResponse::BadRequest().body("create_library require `path` field being existed on the disk of server.");
            }
            let lib = Library::create(path.to_string(),
                                      qs_get_string("library_name").unwrap(),
                                      qs_get_string("master_name"),
                                      qs_get_string("media_folder"),
            );
            let lib = match lib {
                Ok(v) => v,
                Err(e) => return HttpResponse::BadRequest().body(format!("create_library failed, {}", e))
            };
            let uuid = lib.uuid.clone();
            async {
                let mut opened_library = context.opened_libraries.lock().await;
                opened_library.insert(lib.uuid, lib);
            }.await;
            HttpResponse::Ok().body(format!(r#"{{"api": "create_library", "status": "success", "library_uuid": "{}"}}"#, uuid))
        }
        "add_media" => {
            if qs.has("uuid") == false ||
                qs.has("path") == false ||
                qs.has("kind") == false {
                return HttpResponse::BadRequest().body("add_media needs `uuid`, `path`, `kind` field to be present.");
            }
            let uuid = match Uuid::from_str(qs.get("uuid").unwrap()) {
                Ok(v) => v,
                Err(e) => return HttpResponse::BadRequest().body(format!("add_media needs valid `uuid` field. {}", e))
            };
            let path = qs.get("path").unwrap();
            println!("Trying to add media `{}`.", path);
            if !path::PathBuf::from(path).is_file() {
                return HttpResponse::BadRequest().body("add_media needs path to be valid media file on the disk");
            };
            let mut opened_libraries = context.opened_libraries.lock().await;
            let mut library = opened_libraries.get_mut(&uuid);
            if let None = library {
                return HttpResponse::BadRequest().body(format!("library with uuid `{}` not opened.", uuid.to_string()));
            }
            let mut library = library.unwrap();
            let id = library.add_media(
                path.to_string(),
                MediaType::from(qs.get("kind").unwrap()),
                qs_get_string("sub_kind"),
                qs_get_string("kind_addition"),
                qs_get_string("caption"),
                qs_get_string("comment"));
            drop(opened_libraries);
            let id = match id {
                Ok(id) => id,
                Err(e) => return HttpResponse::BadRequest().body(format!("Failed add_media {}", e))
            };
            HttpResponse::Ok().body(format!(r#"{{"api": "add_media", "status": "success", "library_uuid": "{}", "media_id": {}}}"#, uuid.to_string(), id))
        }
        "remove_media" => {
            if !qs.has("id") || !qs.has("uuid") {
                return HttpResponse::BadRequest().body("remove_media needs field `id` and `uuid`");
            }
            let id = qs.get("id").unwrap();
            let id: u64 = match id.parse() {
                Ok(v) => v,
                Err(e) => return HttpResponse::BadRequest().body(format!("remove_media needs id to be a valid number. {}", e))
            };
            HttpResponse::Ok().body("ff")
        }
        _ => HttpResponse::BadRequest().body("Not a valid api request."),
    }
}