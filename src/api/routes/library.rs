use super::*;

use log::info;
use std::{io, path::PathBuf};
use tokio::sync::Mutex;

use actix_web::get;
use shiromana_rs::library::{Library, LibraryFeatures};
use shiromana_rs::media::{Media, MediaType};
use shiromana_rs::misc::{Error as LibError, Uuid};

generate_api_broker!(library_open, get, "library/open",
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<ServerMessage>,
    {
        let path: String = get_param(&params, "path")?;
        if !PathBuf::from(&path).is_dir() {
            return Err(Error::NotExisted {
                got: path,
                field: "path".to_string(),
                expect: "Folder".to_string(),
            });
        }
        let lib = Library::open(path)?;
        let lib_uuid = lib.uuid.clone();
        {
            let mut opened_libraries = opened_libraries.lock().await;
            let uuid = lib.uuid.clone();
            opened_libraries.insert(uuid, lib);
        }
        Ok(ServerMessage {
            library: Some(lib_uuid),
            ..msg
        })
});

generate_api_broker!(library_close, get, "library/close",
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<ServerMessage>,
    {
        if let Some(library_uuid) = library_uuid{
            let mut opened_libraries = opened_libraries.lock().await;
            if opened_libraries.contains_key(&library_uuid) {
                if let Some(v) = opened_libraries.remove(&library_uuid) {
                    drop(v);
                    Ok(ServerMessage {
                        library: Some(library_uuid),
                        ..msg
                    })
                } else {
                    Ok(msg.with_single_error("library", "Cannot remove opened library.", Some(library_uuid), None))
                }
            } else {
                Err(Error::LibraryNotOpened(library_uuid))
            }
        }else {
            Err(Error::NoParam("library".to_string()))
        }
});

generate_api_broker!(library_create, get, "library/create",
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<ServerMessage>,
    {
        Ok(msg)
});

register_services!(library_open, library_close);
