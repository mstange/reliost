use actix_web::{web, Responder};
use wholesym::SymbolManager;

use std::sync::Arc;

#[tracing::instrument(name = "Asm v1", skip(contents, symbol_manager))]
pub async fn asm_v1(
    contents: web::Bytes,
    symbol_manager: web::Data<Arc<SymbolManager>>,
) -> impl Responder {
    let request_json = std::str::from_utf8(&contents).unwrap();
    let response_json = symbol_manager
        .get_ref()
        .query_json_api("/asm/v1", request_json)
        .await;
    web::Json(response_json)
}
