use std::path::PathBuf;

use config::ConfigError;
use serde::Deserialize;

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
