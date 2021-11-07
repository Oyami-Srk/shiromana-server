mod broker;
mod error;
mod message;

use actix_web::web;

pub fn service_config(cfg: &mut web::ServiceConfig) {
    // cfg.service(web::scope("test").service(api_prehandler));
    cfg.service(broker::get_media);
}
