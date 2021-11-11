use super::*;

use actix_web::{get, post};
use mime::Mime;
use mime_sniffer::MimeTypeSniffer;
use shiromana_rs::library::Library;
use shiromana_rs::media::{Media, MediaType};
use std::io::prelude::*;

generate_api_broker!(media_get, get, "media/get",
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<ServerMessage>,
    {
        let library_uuid = library_uuid
            .ok_or_else(|| Error::NoParam("Library".into()))?;
        let id = get_param(&params, "id")?;
        let media = take_mutex!(opened_libraries, {
            let lib = opened_libraries.get(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.get_media(id)
        })?;
        Ok(msg.with_media(id).with_result(serde_json::to_string(&media)?))
});

generate_api_broker!(media_add, post, "media/add",
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<ServerMessage>,
    {
        let library_uuid = library_uuid
            .ok_or_else(|| Error::NoParam("Library".into()))?;
        let path: String = get_param(&params, "path")?;
        if !path::PathBuf::from(&path).is_file() {
            return Err(Error::NotExisted{
                got: path,
                expect: "File".into(),
                field: "path".into()
            });
        }

        let kind = match get_param_option::<String>(&params, "type")? {
            Some(v) => v,
            None => {
                // guess format
                let mut file = std::fs::File::open(&path)?;
                let mut buffer = [0; 64]; // 64 bytes is enough I guess
                file.read(&mut buffer)?;
                let mime_type = match buffer.sniff_mime_type() {
                    Some(s) => s,
                    None => return Ok(
                        msg.with_single_error(
                            "Media",
                            "Cannot guess file type, please provide parameter `type`.",
                            Some(library_uuid),
                            None
                        ))
                }.parse::<Mime>().unwrap(); // not risky unwrap, believe sniffer
                match mime_type.type_() {
                    mime::IMAGE => "image",
                    mime::TEXT => "text",
                    mime::AUDIO => "audio",
                    mime::VIDEO => "video",
                    _ => "other"
                }.into()
            }
        };
        let id = take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.add_media(
                path.clone(),
                MediaType::from_str(kind.as_str())?,
                get_param_option(&params, "sub_type")?,
                get_param_option(&params, "type_addition")?,
                get_param_option(&params, "caption")?,
                get_param_option(&params, "comment")?
            )
        })?;
        if get_param_bool(&params, "delete")? == true {
            // remove original file
            if let Err(e) = std::fs::remove_file(&path) {
                return Ok(msg.with_single_error_but_partial_success(
                    "Media",
                    format!("Failed to remove original file `{}` due to {}.", path, e),
                    Some(library_uuid),
                    Some(id)
                ))
            }
        }
        Ok(msg.with_media(id))
});

generate_api_broker!(media_remove, post, "media/remove",
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<ServerMessage>,
    {
        let library_uuid = library_uuid
            .ok_or_else(|| Error::NoParam("Library".into()))?;
        let id = get_param(&params, "id")?;
        take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.remove_media(id)?;
        });
        Ok(msg.with_media(id))
});

generate_api_broker!(media_update, post, "media/update",
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<ServerMessage>,
    {
        let library_uuid = library_uuid
            .ok_or_else(|| Error::NoParam("Library".into()))?;
        // let id = get_param(&params, "id")?;
        let media = get_param::<String>(&params, "media")?;
        let mut media: Media = serde_json::from_str(media.as_str())?;
        let id = media.id;

        take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.update_media(&mut media)?;
        });

        Ok(msg.with_media(id))
});

register_services!(media_get, media_add, media_remove, media_update);
