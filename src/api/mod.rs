mod error;
mod message;
mod routes;

use actix_web::{
    dev::HttpServiceFactory,
    http::header::HttpDate,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};

pub fn service_config(cfg: &mut web::ServiceConfig) {
    // cfg.service(broker::media_get);
    routes::services(cfg);
}
