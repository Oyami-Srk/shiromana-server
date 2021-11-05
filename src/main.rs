#![allow(dead_code, unused)]
#![feature(async_closure)]

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use actix_web::middleware::Logger;
use actix_web::{get, route, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use clap::{App as clapApp, Arg};
use env_logger::Env;
use log::info;
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
    HttpResponse::Ok().body(format!(
        "Hello world. This is shiromana-server version {}\n",
        env!("CARGO_PKG_VERSION")
    ))
}

#[route(
    "/{api_name}",
    method = "GET",
    method = "POST",
    method = "PUT",
    method = "DELETE"
)]
async fn api_prehandler(
    req: HttpRequest,
    web::Path(api_name): web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    // HttpResponse::Ok().body(format!("Hello from {}, with query param: {}", api_name, req.query_string()))
    info!(
        "Request api {} with method {}.",
        api_name,
        req.method().as_str()
    );
    api::dispatcher(&api_name, &data, req).await
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // get arguments
    let matches = clapApp::new("Shiromana Server")
        .version("0.1.1")
        .about("Shiromana Media Manager Http Server of APIs.")
        .author("Shiroko <hhx.xxm@gmail.com>")
        .arg(
            Arg::with_name("host")
                .short("H")
                .long("host")
                .value_name("IP:PORT")
                .help("Server's listen address with port.")
                .takes_value(true)
                .multiple(false)
                .default_value("127.0.0.1:22110")
                .validator(|v| {
                    if v.parse::<SocketAddr>().is_err() {
                        return Err("Host must be like `127.0.0.1:22110`".to_string());
                    }
                    return Ok(());
                }),
        )
        .get_matches();
    // setup logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stdout)
        .write_style(env_logger::WriteStyle::Always)
        .init();
    // setup libraries shared Mutex
    let opened_libraries = Arc::new(Mutex::new(HashMap::new()));
    let clone_of_opened_libraries = opened_libraries.clone();
    // start server
    // let server_config = ServerConfig::default();
    // let listen_addr = SocketAddr::new(server_config.host, server_config.port);
    let listen_addr: SocketAddr = matches.value_of("host").unwrap().parse().unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(AppState {
                opened_libraries: opened_libraries.clone(),
            })
            .service(root)
            .service(web::scope("/api").configure(api_service_config))
    })
    //.workers(1)
    .bind(listen_addr)?
    .run()
    .await;

    info!("Http server is shutting down.");
    info!("Running clean up routine");
    {
        let mut opened_libraries = clone_of_opened_libraries.lock().await;
        opened_libraries.clear();
    }
    info!("Bye, see you next time~");
    Ok(())
}
