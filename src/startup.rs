use std::{net::TcpListener, sync::Arc};

use crate::configuration::Settings;
use crate::routes::{asm_v1, greet, health_check, symbolicate_v5};
use crate::symbol_manager::create_symbol_manager;
use actix_cors::Cors;
use actix_web::{dev::Server, web, App, HttpServer};
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, settings: Settings) -> Result<Server, std::io::Error> {
    let symbol_manager = Arc::new(create_symbol_manager(settings));
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "OPTION"])
            .allow_any_header();
        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default())
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
