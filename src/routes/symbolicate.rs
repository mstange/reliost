use actix_web::{web, HttpResponse, Responder};
use std::{
    io::{BufWriter, Write},
    sync::Arc,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use wholesym::SymbolManager;

use crate::channel_writer::BlockingChannelWriter;

const CHUNK_SIZE: usize = 64 * 1024;

#[tracing::instrument(name = "Symbolicate v5", skip(contents, symbol_manager))]
pub async fn symbolicate_v5(
    contents: web::Bytes,
    symbol_manager: web::Data<Arc<SymbolManager>>,
) -> impl Responder {
    let request_json = std::str::from_utf8(&contents).unwrap();
    let response_json = symbol_manager
        .get_ref()
        .query_json_api("/symbolicate/v5", request_json)
        .await;

    let (tx, rx) = mpsc::channel(4);
    tokio::task::spawn_blocking(move || {
        let mut writer = BufWriter::with_capacity(CHUNK_SIZE, BlockingChannelWriter::new(tx));
        serde_json::to_writer(&mut writer, &response_json).unwrap();
        writer.flush().unwrap();
        drop(writer); // This ends the response.
        drop(response_json); // deallocations after response end
    });

    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(ReceiverStream::new(rx))
}
