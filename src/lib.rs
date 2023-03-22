use std::{net::TcpListener, path::PathBuf, sync::Arc};

use actix_cors::Cors;
use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};
use config::ConfigError;
use serde::Deserialize;
use wholesym::{SymbolManager, SymbolManagerConfig};

#[derive(Deserialize)]
pub struct Settings {
    pub server: ServerSettings,
    pub symbols: Option<SymbolSettings>,
}

#[derive(Deserialize)]
pub struct ServerSettings {
    pub port: u16,
}

#[derive(Deserialize)]
pub struct SymbolSettings {
    pub breakpad: Option<BreakpadSymbolSettings>,
    pub windows: Option<WindowsSymbolSettings>,
}

#[derive(Deserialize)]
pub struct BreakpadSymbolSettings {
    #[serde(default)]
    pub servers: Vec<String>,

    pub cache_dir: PathBuf,
    pub symindex_dir: Option<PathBuf>,
}

#[derive(Deserialize)]
pub struct WindowsSymbolSettings {
    #[serde(default)]
    pub servers: Vec<String>,

    pub cache_dir: PathBuf,
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.toml",
            config::FileFormat::Toml,
        ))
        .build()?;
    settings.try_deserialize::<Settings>()
}

fn create_symbol_manager(settings: Settings) -> SymbolManager {
    let mut config = SymbolManagerConfig::default().verbose(true);
    if let Some(symbols) = settings.symbols {
        if let Some(breakpad) = symbols.breakpad {
            if breakpad.servers.is_empty() {
                config = config.breakpad_symbols_dir(breakpad.cache_dir);
            } else {
                for server_url in breakpad.servers {
                    config = config.breakpad_symbols_server(server_url, breakpad.cache_dir.clone());
                }
            }
            if let Some(symindex_dir) = breakpad.symindex_dir {
                config = config.breakpad_symindex_cache_dir(symindex_dir);
            }
        }
        if let Some(windows) = symbols.windows {
            for server_url in windows.servers {
                config = config.windows_symbols_server(server_url, windows.cache_dir.clone());
            }
        }
    }
    SymbolManager::with_config(config)
}

async fn greet() -> impl Responder {
    "Hello world!".to_string()
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

async fn symbolicate_v5(
    contents: web::Bytes,
    symbol_manager: web::Data<Arc<SymbolManager>>,
) -> impl Responder {
    let request_json = std::str::from_utf8(&contents).unwrap();
    symbol_manager
        .get_ref()
        .query_json_api("/symbolicate/v5", request_json)
        .await
}

async fn asm_v1(
    contents: web::Bytes,
    symbol_manager: web::Data<Arc<SymbolManager>>,
) -> impl Responder {
    let request_json = std::str::from_utf8(&contents).unwrap();
    symbol_manager
        .get_ref()
        .query_json_api("/asm/v1", request_json)
        .await
}

pub fn run(listener: TcpListener, settings: Settings) -> Result<Server, std::io::Error> {
    let symbol_manager = Arc::new(create_symbol_manager(settings));
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "OPTION"])
            .allow_any_header();
        App::new()
            .wrap(cors)
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
