use actix_web::{error, http::header::ContentType, HttpResponse, Responder};
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VersionError {
    #[error("The version file could not be found.")]
    VersionFileNotFound,
    #[error("An error occurred while retrieving the version file: {error}")]
    Other { error: io::Error },
}

// Use the default implementation for `error_response()` method which returns 500.
impl error::ResponseError for VersionError {}

impl From<io::Error> for VersionError {
    fn from(err: io::Error) -> VersionError {
        match err.kind() {
            io::ErrorKind::NotFound => VersionError::VersionFileNotFound,
            _ => VersionError::Other { error: err },
        }
    }
}

/// "Respond to `/__version__` with the contents of /app/version.json."
#[tracing::instrument(name = "Get version")]
pub async fn version() -> Result<HttpResponse, VersionError> {
    // Our build.rs script places version.json in $CARGO_MANIFEST_DIR,
    // i.e. next to Cargo.toml.
    // If we're run via `cargo run`, $CARGO_MANIFEST_DIR is set to that
    // directory, otherwise we check the working directory instead.
    let out_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map_or_else(|| env::current_dir().unwrap(), PathBuf::from);
    let path = Path::new(&out_dir).join("version.json");

    // This is a very small file, that's why it's not a problem to directly read it.
    // We could cache it in the future if we need to.
    match fs::read_to_string(path) {
        Ok(contents) => {
            let response = HttpResponse::Ok()
                .content_type(ContentType::json())
                .body(contents);
            Ok(response)
        }
        Err(err) => {
            tracing::error!("Failed to read version.json file to string. Error: {}", err);
            Err(err.into())
        }
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
