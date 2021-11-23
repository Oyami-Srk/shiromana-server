use super::*;

use actix_web::{get, post};
use shiromana_rs::library::Library;

generate_api_broker!(series_create, post, "series/create",
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
        let series = take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.create_series(get_param(&params, "caption")?,
                get_param_option(&params, "comment")?)
        })?;
        Ok(msg.with_result(series).with_format("uuid"))
});

generate_api_broker!(series_delete, post, "series/delete",
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
            lib.delete_series(&get_param(&params, "series")?)?;
        });
        Ok(msg)
});

generate_api_broker!(series_add_media, post, "series/add_media",
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
            lib.add_to_series(
                get_param(&params, "media")?,
                &get_param(&params, "series")?,
                get_param_option(&params, "no")?,
                get_param_bool(&params, "unsorted")?
            )?;
        });
        Ok(msg)
});

generate_api_broker!(series_remove_media, post, "series/remove_media",
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
            lib.remove_from_series(
                get_param(&params, "media")?,
                &get_param(&params, "series")?,
            )?;
        });
        Ok(msg)
});

generate_api_broker!(series_update_no, post, "series/update_no",
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
            lib.update_series_no(
                get_param(&params, "media")?,
                &get_param(&params, "series")?,
                get_param(&params, "no")?,
                get_param_bool(&params, "insert")?
            )?;
        });
        Ok(msg)
});

generate_api_broker!(series_trim_no, post, "series/trim_no",
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
            lib.trim_series_no(
                &get_param(&params, "series")?,
            )?;
        });
        Ok(msg)
});

register_services!(
    series_create,
    series_delete,
    series_add_media,
    series_remove_media,
    series_update_no,
    series_trim_no
);
