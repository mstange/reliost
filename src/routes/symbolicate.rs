use actix_web::{
    http::header::{self, ContentEncoding},
    mime, web, HttpResponse, Responder,
};
use flate2::{write::GzEncoder, Compression};
use std::{
    io::{BufWriter, Write},
    sync::Arc,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use wholesym::SymbolManager;

use crate::{channel_writer::BlockingChannelWriter, double_buffered_pipe::RemoteBufWriter};

const CHUNK_SIZE: usize = 64 * 1024;
const GZIP_COMPRESSION_LEVEL: u32 = 2; // not tweaked

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
        let writer = BufWriter::with_capacity(CHUNK_SIZE, BlockingChannelWriter::new(tx));
        let writer = GzEncoder::new(writer, Compression::new(GZIP_COMPRESSION_LEVEL));
        let mut writer = RemoteBufWriter::with_capacity(CHUNK_SIZE, writer);
        serde_json::to_writer(&mut writer, &response_json).unwrap();
        writer.flush().unwrap();
        drop(writer); // This ends the response.
        drop(response_json); // deallocations after response end
    });

    HttpResponse::Ok()
        .content_type(mime::APPLICATION_JSON)
        .append_header((header::CONTENT_ENCODING, ContentEncoding::Gzip))
        .streaming(ReceiverStream::new(rx))
}
