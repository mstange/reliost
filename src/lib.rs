pub mod configuration;
pub mod symbol_manager;

use std::{net::TcpListener, sync::Arc};

use crate::configuration::Settings;
use crate::symbol_manager::create_symbol_manager;
use actix_cors::Cors;
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use wholesym::SymbolManager;

async fn greet() -> impl Responder {
    "Hello world!".to_string()
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

async fn symbolicate_v5(
    contents: web::Bytes,
    symbol_manager: web::Data<Arc<SymbolManager>>,
) -> impl Responder {
    let request_json = std::str::from_utf8(&contents).unwrap();
    symbol_manager
        .get_ref()
        .query_json_api("/symbolicate/v5", request_json)
        .await
}

async fn asm_v1(
    contents: web::Bytes,
    symbol_manager: web::Data<Arc<SymbolManager>>,
) -> impl Responder {
    let request_json = std::str::from_utf8(&contents).unwrap();
    symbol_manager
        .get_ref()
        .query_json_api("/asm/v1", request_json)
        .await
}

pub fn run(listener: TcpListener, settings: Settings) -> Result<Server, std::io::Error> {
    let symbol_manager = Arc::new(create_symbol_manager(settings));
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "OPTION"])
            .allow_any_header();
        App::new()
            .wrap(cors)
            .route("/", web::get().to(greet))
            .route("/health_check", web::get().to(health_check))
            .route("/symbolicate/v5", web::post().to(symbolicate_v5))
            .route("/asm/v1", web::post().to(asm_v1))
            .app_data(web::Data::new(symbol_manager.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
