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

use super::error::{Error, Result};
use super::AppState;
