use actix_web::{web, Responder};
use std::sync::Arc;
use wholesym::SymbolManager;

#[tracing::instrument(name = "Symbolicate v5", skip(contents, symbol_manager))]
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
