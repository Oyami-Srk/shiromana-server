#![allow(dead_code, unused)]

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use actix_web::{App, get, HttpRequest, HttpResponse, HttpServer, post, Responder, web};
use shiromana_rs::library::Library;
use shiromana_rs::misc::Uuid;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

mod api;

pub struct AppState {
    pub opened_libraries: Arc<Mutex<HashMap<Uuid, Library>>>,
}

struct ServerConfig {
    host: IpAddr,
    port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: IpAddr::from([127, 0, 0, 1]),
            port: 22110,
        }
    }
}

#[get("/")]
async fn root(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("Hello root.")
}

#[get("/{api_name}")]
async fn api_prehandler(req: HttpRequest, web::Path(api_name): web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    // HttpResponse::Ok().body(format!("Hello from {}, with query param: {}", api_name, req.query_string()))
    api::dispatcher(&api_name, &data, req).await
}

fn api_service_config(cfg: &mut web::ServiceConfig) {
    cfg
        .service(
            api_prehandler
        );
}

struct TestDrop;

impl Drop for TestDrop {
    fn drop(&mut self) {
        println!("Dropped.");
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opened_libraries = Arc::new(Mutex::new(HashMap::new()));
    let clone_of_opened_libraries = opened_libraries.clone();
    // start server
    let server_config = ServerConfig::default();
    let listen_addr = SocketAddr::new(server_config.host, server_config.port);

    HttpServer::new(move || {
        App::new()
            .data(
                AppState {
                    opened_libraries: opened_libraries.clone(),
                }
            )
            .service(root)
            .service(
                web::scope("/api")
                    .configure(api_service_config)
            )
    })
        //.workers(1)
        .bind(listen_addr)?
        .run()
        .await;
    println!("Http server is shutting down.");
    println!("Running clean up routine");
    {
        let mut opened_libraries = clone_of_opened_libraries.lock().await;
        opened_libraries.clear();
    }
    println!("Bye, see you next time~");
    Ok(())
}
