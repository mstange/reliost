use actix_web::{http::header::ContentType, HttpResponse, Responder};

const VERSION_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/version.json"));

/// "Respond to `/__version__` with the contents of version.json."
pub async fn version() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(VERSION_JSON)
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
