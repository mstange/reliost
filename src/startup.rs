use std::net::TcpListener;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{dev::Server, web, App, HttpServer};
use samply_quota_manager::QuotaManager;
use tracing_actix_web::TracingLogger;

use crate::configuration::Settings;
use crate::routes::{asm_v1, greet, heartbeat, lbheartbeat, symbolicate_v5, version};
use crate::symbol_manager::create_symbol_manager_and_quota_manager;

#[tracing::instrument(skip_all)]
pub fn run(
    listener: TcpListener,
    settings: Settings,
) -> Result<(Server, Option<QuotaManager>), std::io::Error> {
    let (symbol_manager, quota_manager) = create_symbol_manager_and_quota_manager(settings);
    let app_data = web::Data::new(Arc::new(symbol_manager));
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "OPTION"])
            .allow_any_header()
            .max_age(86400);
        App::new()
            .wrap(cors)
            .wrap(TracingLogger::default())
            .route("/", web::get().to(greet))
            .route("/symbolicate/v5", web::post().to(symbolicate_v5))
            .route("/asm/v1", web::post().to(asm_v1))
            // Dockerflow requirements. See:
            // https://github.com/mozilla-services/Dockerflow#containerized-app-requirements
            .route("/__version__", web::get().to(version))
            .route("/__heartbeat__", web::get().to(heartbeat))
            .route("/__lbheartbeat__", web::get().to(lbheartbeat))
            .app_data(app_data.clone())
            .app_data(web::PayloadConfig::new(100 * 1000 * 1000)) // 100 MB
    })
    .listen(listener)?
    .run();
    Ok((server, quota_manager))
}
