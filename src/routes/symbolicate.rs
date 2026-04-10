use actix_web::{
    http::{
        header::{self, ContentEncoding},
        StatusCode,
    },
    mime, web, HttpResponse, Responder,
};
use flate2::{write::GzEncoder, Compression};
use std::{
    io::{BufWriter, Write},
    sync::Arc,
};
use wholesym::SymbolManager;

use crate::{channel_writer::writer_with_stream, double_buffered_pipe::RemoteBufWriter};

const CHUNK_SIZE: usize = 64 * 1024;
const GZIP_COMPRESSION_LEVEL: u32 = 2; // not tweaked

#[tracing::instrument(name = "Symbolicate v5", skip(request_json, symbol_manager))]
pub async fn symbolicate_v5(
    request_json: String,
    symbol_manager: web::Data<Arc<SymbolManager>>,
) -> impl Responder {
    let response_json = symbol_manager
        .get_ref()
        .query_json_api("/symbolicate/v5", &request_json)
        .await;

    let status = StatusCode::from_u16(response_json.http_status())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let (writer, stream) = writer_with_stream(vec![
        Vec::with_capacity(CHUNK_SIZE),
        Vec::with_capacity(CHUNK_SIZE),
    ]);
    tokio::task::spawn_blocking(move || {
        let writer = BufWriter::with_capacity(CHUNK_SIZE, writer);
        let writer = GzEncoder::new(writer, Compression::new(GZIP_COMPRESSION_LEVEL));
        let mut writer = RemoteBufWriter::with_capacity(CHUNK_SIZE, writer);
        serde_json::to_writer(&mut writer, &response_json).unwrap();
        writer.flush().unwrap();
        drop(writer); // This ends the response.
        drop(response_json); // deallocations after response end
    });

    HttpResponse::build(status)
        .content_type(mime::APPLICATION_JSON)
        .append_header((header::CONTENT_ENCODING, ContentEncoding::Gzip))
        .streaming(stream)
}
