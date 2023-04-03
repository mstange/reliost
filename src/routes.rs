use actix_web::{http::header::ContentType, web, HttpResponse, Responder};
use std::{env, fs, path::Path, sync::Arc};
use wholesym::SymbolManager;

pub async fn greet() -> impl Responder {
    "Hello world!".to_string()
}

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub async fn symbolicate_v5(
    contents: web::Bytes,
    symbol_manager: web::Data<Arc<SymbolManager>>,
) -> impl Responder {
    let request_json = std::str::from_utf8(&contents).unwrap();
    symbol_manager
        .get_ref()
        .query_json_api("/symbolicate/v5", request_json)
        .await
}

pub async fn asm_v1(
    contents: web::Bytes,
    symbol_manager: web::Data<Arc<SymbolManager>>,
) -> impl Responder {
    let request_json = std::str::from_utf8(&contents).unwrap();
    symbol_manager
        .get_ref()
        .query_json_api("/asm/v1", request_json)
        .await
}

/// "Respond to `/__version__` with the contents of /app/version.json."
pub async fn version() -> impl Responder {
    // Cargo sets the OUT_DIR to appropriate directory for debug and leaves it empty for release.
    // Build script places it to the root directory if it's release.
    let out_dir = env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string());
    let path = Path::new(&out_dir).join("version.json");

    // This is a very small file, that's why it's not a problem to directly read it.
    // We could cache it in the future if we need to.
    match fs::read_to_string(path) {
        Ok(contents) => HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(contents),
        Err(err) => HttpResponse::from_error(err),
    }
}

/// "Respond to `/__heartbeat__` with a HTTP 200 or 5xx on error. This should check
/// backing services like a database for connectivity and may respond with the
/// status of backing services and application components as a JSON payload."
pub async fn heartbeat() -> impl Responder {
    HttpResponse::Ok()
}

/// "Respond to `/__lbheartbeat__` with an HTTP 200. This is for load balancer
/// checks and should not check backing services."
pub async fn lbheartbeat() -> impl Responder {
    HttpResponse::Ok()
}
