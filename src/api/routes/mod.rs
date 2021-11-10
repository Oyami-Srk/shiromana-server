mod library;

pub(crate) use super::super::AppState;
pub(crate) use super::error::{Error, Result};
pub(crate) use super::message::{ServerApiStatus, ServerMessage};
pub(crate) use actix_web::{web, HttpRequest, HttpResponse, Responder};
pub(crate) use qstring::QString;
pub(crate) use shiromana_rs::misc::Uuid;
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

macro_rules! generate_api_broker {
    ($name: ident, $method: ident, $route: expr, ($($arg:ident:$typ:ty),*) -> $rt:ty, $body: block) => {
        paste::paste!{
            async fn [<perform_ $name>](
                /*
                library_uuid: Option<Uuid>,
                opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
                action: &str,
                params: QString,
                msg: ServerMessage
                */
                $($arg:$typ,)*
            ) -> $rt // Result<ServerMessage>
            $body
        }

        #[$method($route)]
        pub async fn $name(
            req: HttpRequest,
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

            let result = paste::paste!([<perform_ $name>])(library_uuid, &data.opened_libraries, stringify!($name), qs, server_msg.clone()).await;

            match result {
                Ok(v) => {
                    if let(ServerApiStatus::Success) = v.status {
                        HttpResponse::Ok().body(v.to_json_string())
                    } else {
                        HttpResponse::BadRequest().body(v.to_json_string())
                    }
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

pub(crate) use generate_api_broker;
pub(crate) use register_services;

pub fn services(cfg: &mut web::ServiceConfig) {
    library::services(cfg);
}
