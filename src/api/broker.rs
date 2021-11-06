use std::collections::HashMap;
use std::process;
use std::str::FromStr;
use std::{io, path};

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

macro_rules! generate_api_broker {
    ($name: ident, $method: ident) => {
        #[$method("$name")]
        pub fn $name() -> impl Responder {
            match perform_action() {
                Ok(v) => v.into(),
                Err(e) => ServerMessage {
                    
                }.into(),
            }
        }
    };
}

fn perform_action(library: &Uuid, action: &str, param: QString) -> Result<ServerMessage> {
    let msg = ServerMessage::default();

    Ok(msg)
}

generate_api_broker!(get_media, get)
