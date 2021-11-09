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

register_services!(library_open);