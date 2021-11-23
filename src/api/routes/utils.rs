use std::io::Read;

use super::*;

use actix_files::NamedFile;
use actix_web::{get, post};
use mime;
use mime_sniffer::MimeTypeSniffer;
use shiromana_rs::library::Library;

generate_api_broker!(utils_make_thumbnail, post, "utils/make_thumbnail",
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
        let buffer = take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.make_thumbnail(get_param(&params, "media")?)
        }).recv()??;
        Ok(msg.with_result(base64::encode(buffer)).with_format("base64"))
});

generate_api_broker!(utils_get_thumbnail, get, "utils/get_thumbnail",
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
        let buffer = take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&library_uuid)
                .ok_or_else(|| Error::LibraryNotOpened(library_uuid))?;
            lib.get_thumbnail(get_param(&params, "media")?)
        }).recv()??;
        Ok(msg.with_result(base64::encode(buffer)).with_format("base64"))
});

generate_api_broker!(utils_get_thumbnail_b, get, "{lib}/{media}/thumbnail",
    (
        lib: Uuid,
        media: u64
    ),
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<HttpResponse>,
    {
        let buffer = take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&lib)
                .ok_or_else(|| Error::LibraryNotOpened(lib))?;
            lib.get_thumbnail(media)
        }).recv()??;
        Ok(HttpResponse::Ok().body(buffer))
});

generate_api_broker!(utils_get_media_b, get, "{lib}/{media}/media",
    (
        lib: Uuid,
        media: u64
    ),
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<NamedFile>,
    {
        let media = take_mutex!(opened_libraries, {
            let mut lib = opened_libraries.get_mut(&lib)
                .ok_or_else(|| Error::LibraryNotOpened(lib))?;
            lib.get_media(media)?
        });
        let filepath = media.filepath;

        let mut file = std::fs::File::open(&filepath)?;
        let mut buffer = [0; 64]; // 64 bytes is enough I guess
        file.read(&mut buffer)?;
        let mime_type = match buffer.sniff_mime_type() {
            Some(s) => Some(s.parse::<mime::Mime>().unwrap()),
            None => None
        };

        let f = NamedFile::open(filepath)?.disable_content_disposition();
        let f = match mime_type {
            Some(t) => f.set_content_type(t),
            None => f
        };
        Ok(f)
});

register_services!(
    utils_make_thumbnail,
    utils_get_thumbnail,
    utils_get_thumbnail_b,
    utils_get_media_b
);
