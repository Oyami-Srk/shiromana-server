use log::info;
use std::collections::HashMap;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::{io, path};
use tokio::sync::Mutex;

use actix_web::error::PayloadError::Http2Payload;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use qstring::QString;
use serde::{Deserialize, Serialize};
use shiromana_rs::library::{Library, LibraryFeatures};
use shiromana_rs::media::{Media, MediaType};
use shiromana_rs::misc::{Error as LibError, Uuid};
use tokio::sync::mpsc::Sender;

use super::super::AppState;
use super::error::{Error, Result};
use super::message::{ServerApiStatus, ServerMessage};
use paste::paste;
use std::stringify;

fn get_param<T>(params: &QString, key: &str) -> Result<T>
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

fn get_param_option<T>(params: &QString, key: &str) -> Result<Option<T>>
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
    ($name: ident, $method: ident, $route: expr, $body: expr) => {
        #[$method($route)]
        pub async fn $name(
            req: HttpRequest,
            data: web::Data<AppState>,
        ) -> impl Responder {
            let qs = QString::from(req.query_string());
            let server_msg = ServerMessage {
                api: stringify!($name).to_string(),
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

            let func: fn(
                Option<Uuid>,
                &Arc<Mutex<HashMap<Uuid, Library>>>,
                &str,
                QString,
                ServerMessage
            ) -> Result<ServerMessage> = $body;

            match func(library_uuid, &data.opened_libraries, stringify!($name), qs, server_msg.clone()) {
                Ok(v) => HttpResponse::Ok().body(v.to_json_string()),
                Err(e) => HttpResponse::BadRequest().body(
                    server_msg.with_single_error("action", e.to_string(), library_uuid, None).to_json_string()
                )
            }
        }
    };
}

fn perform_action(
    library_uuid: Option<Uuid>,
    opened_libraries: &Arc<Mutex<HashMap<Uuid, Library>>>,
    action: &str,
    params: QString,
    msg: ServerMessage,
) -> Result<ServerMessage> {
    info!("Nice, a request to {}", action);
    Ok(msg)
}

generate_api_broker!(
    get_media,
    get,
    "get_media",
    |library_uuid, opened_libraries, action, params, msg| {
        let msg = ServerMessage {
            api: "hitest".to_string(),
            ..msg
        };
        Ok(msg)
    }
);
