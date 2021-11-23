mod library;
mod media;
mod series;
mod tag;
mod utils;

pub(crate) use super::super::AppState;
pub(crate) use super::error::{Error, Result};
pub(crate) use super::message::{ServerApiStatus, ServerMessage};
use actix_files::HttpRange;
pub(crate) use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
pub(crate) use qstring::QString;
use serde::Deserialize;
use shiromana_rs::library::{LibraryFeatures, LibraryMetadata, LibrarySummary};
pub(crate) use shiromana_rs::{library::Library, misc::Uuid};
pub(crate) use std::collections::HashMap;
pub(crate) use std::str::FromStr;
pub(crate) use std::stringify;
pub(crate) use std::sync::Arc;
pub(crate) use std::{io, path};
pub(crate) use tokio::sync::Mutex;

pub fn get_param<T>(params: &QString, key: &str) -> Result<T>
where
    T: FromStr,
{
    match params.get(key) {
        Some(v) => match v.parse::<T>() {
            Ok(v) => Ok(v),
            Err(e) => Err(Error::ParamInvalid {
                got: v.to_string(),
                field: key.to_string(),
                expect: std::any::type_name::<T>().to_string(),
            }),
        },
        None => return Err(Error::NoParam(key.to_string())),
    }
}

pub fn get_param_option<T>(params: &QString, key: &str) -> Result<Option<T>>
where
    T: FromStr,
{
    match get_param(params, key) {
        Ok(v) => Ok(Some(v)),
        Err(Error::NoParam(_)) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_param_bool(params: &QString, key: &str) -> Result<bool> {
    if params.has(key) {
        match params.get(key).unwrap() {
            "false" => Ok(false),
            "true" => Ok(true),
            _ => Err(Error::ParamInvalid {
                got: params.get(key).unwrap().into(),
                expect: "bool".into(),
                field: key.into(),
            }),
        }
    } else {
        Ok(false)
    }
}

macro_rules! expand_or_dash {
    () => {
        _
    };

    ($($a:ident),*) => {
        $($a,)*
    };
}

pub trait IntoResponse {
    fn into_response(self, req: &HttpRequest) -> HttpResponse;
}

impl IntoResponse for HttpResponse {
    fn into_response(self, _: &HttpRequest) -> HttpResponse {
        self
    }
}

impl IntoResponse for ServerMessage {
    fn into_response(self, _: &HttpRequest) -> HttpResponse {
        match self.status {
            ServerApiStatus::Success | ServerApiStatus::PartialSuccess => {
                HttpResponse::Ok().body(self.to_json_string())
            }
            ServerApiStatus::Failed => HttpResponse::BadRequest().body(self.to_json_string()),
        }
    }
}

impl IntoResponse for actix_files::NamedFile {
    fn into_response(self, req: &HttpRequest) -> HttpResponse {
        match self.into_response(req) {
            Ok(v) => v,
            Err(e) => HttpResponse::BadRequest().body(format!("Error while opening file: {}", e)),
        }
    }
}

macro_rules! generate_api_broker {
    ($name: ident, $method: ident, $route: expr, ($($arg:ident:$typ:ty),*) -> Result<$rt:ty>, $body: block) => {
        generate_api_broker!(
            $name,
            $method,
            $route, (), (
                $($arg:$typ),*
            ) -> Result<$rt>,
            $body);
    };

    ($name: ident, $method: ident, $route: expr, ($($path_arg:ident:$path_ty:ty),*),($($arg:ident:$typ:ty),*) -> Result<$rt:ty>, $body: block) => {
        paste::paste!{
            async fn [<perform_ $name>](
                $($arg:$typ,)*
                $($path_arg:$path_ty,)*
            ) -> Result<$rt>
            $body
        }

        #[$method($route)]
        pub async fn $name(
            req: HttpRequest,
            web::Path(($($path_arg,)*)): web::Path<($($path_ty,)*)>,
            data: web::Data<AppState>,
        ) -> impl Responder {
            let qs = QString::from(req.query_string());
            let server_msg = ServerMessage {
                api: $route.to_string(),//stringify!($name).to_string(),
                is_preety: match qs.get("pretty").unwrap_or("true").to_lowercase().as_str() {
                    "false" => false,
                    "true" => true,
                    _ => true
                },
                ..ServerMessage::default()
            };
            let library_uuid = match qs.get("library") {
                Some(s) => match s.parse::<Uuid>() {
                    Ok(v) => Some(v),
                    Err(e) => {
                        return HttpResponse::BadRequest().body(
                            server_msg.with_single_error(
                                "parameter",
                                format!(
                                    "Parameter `library` is not a valid Uuid identifier. Err: {}", e
                                ),
                                None,
                                None
                            ).to_json_string()
                        )
                    }
                },
                None => None
            };
            let server_msg = ServerMessage{
                library: library_uuid,
                ..server_msg
            };

            let result = paste::paste!([<perform_ $name>])(library_uuid, &data.opened_libraries, stringify!($name), qs, server_msg.clone(), $($path_arg,)*).await;

            match result {
                Ok(v) => {
                    IntoResponse::into_response(v, &req)
                },
                Err(e) => HttpResponse::BadRequest().body(
                    server_msg.with_single_error("action", e.to_string(), library_uuid, None).to_json_string()
                )
            }
        }
    };
}

macro_rules! register_services {
    ( $( $x: ident),* ) => {
        pub fn services(cfg: &mut web::ServiceConfig) {
            $(
                cfg.service($x);
            )*
        }
    };
}

macro_rules! take_mutex {
    ($v:ident, $body: block) => {{
        let mut $v = $v.lock().await;
        $body
    }};
}

pub(crate) use expand_or_dash;
pub(crate) use generate_api_broker;
pub(crate) use register_services;
pub(crate) use take_mutex;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct LibraryInfo {
    path: String,
    metadata: LibraryMetadata,
    summary: LibrarySummary,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ServerStatus {
    server_version: &'static str,
    shiromana_lib_version: &'static str,
    opened_libraries: Vec<LibraryInfo>,
}

generate_api_broker!(status, get, "status",
    (
        library_uuid: Option<Uuid>,
        opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
        action: &str,
        params: QString,
        msg: ServerMessage
    ) -> Result<ServerMessage>,
    {
        let mut libs : Vec<LibraryInfo> = vec![];
        take_mutex!(
            opened_libraries, {
                for (k,v) in opened_libraries.iter() {
                    libs.push(LibraryInfo {
                        path: v.get_path().clone(),
                        metadata: v.get_metadata(),
                        summary: v.get_summary().clone(),
                    });
                }
            }
        );
        let server_status = ServerStatus {
            server_version: env!("CARGO_PKG_VERSION"),
            shiromana_lib_version: crate::versions::SHIROMANA_RS,
            opened_libraries: libs
        };
        Ok(msg.with_serialized_result(&server_status)?.with_format("json"))
});

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(status);
    library::services(cfg);
    media::services(cfg);
    series::services(cfg);
    tag::services(cfg);
    utils::services(cfg);
}
