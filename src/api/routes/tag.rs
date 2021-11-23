use super::*;

use actix_web::{get, post};
use shiromana_rs::library::Library;

generate_api_broker!(tag_create, post, "tag/create",
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
        let tag = take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.create_tag(get_param(&params, "caption")?,
                get_param_option(&params, "comment")?)
        })?;
        Ok(msg.with_result(tag).with_format("uuid"))
});

generate_api_broker!(tag_delete, post, "tag/delete",
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
        take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.delete_tag(get_param(&params, "tag")?)?;
        });
        Ok(msg)
});

generate_api_broker!(tag_add_media, post, "tag/add_media",
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
        take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.add_tag(
                get_param(&params, "media")?,
                &get_param(&params, "tag")?,
            )?;
        });
        Ok(msg)
});

generate_api_broker!(tag_remove_media, post, "tag/remove_media",
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
        take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.remove_tag(
                get_param(&params, "media")?,
                &get_param(&params, "tag")?,
            )?;
        });
        Ok(msg)
});

register_services!(tag_create, tag_delete, tag_add_media, tag_remove_media);
