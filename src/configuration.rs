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
    pub host: String,
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

/// The possible runtime environment for our application.
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}

#[tracing::instrument(name = "Get configuration", skip_all)]
pub fn get_configuration() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.toml", environment.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.toml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .build()?;
    settings.try_deserialize::<Settings>()
}
