use std::net::TcpListener;

use reliost::configuration::get_configuration;
use reliost::logging::{get_subscriber, init_subscriber};
use reliost::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("reliost".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = get_configuration().expect("Failed to read configuration");
    let (server, quota_manager) = run(
        TcpListener::bind((settings.server.host.as_str(), settings.server.port))?,
        settings,
    )?;

    server.await?;

    if let Some(quota_manager) = quota_manager {
        // Shut down the quota manager file deletion thread.
        quota_manager.finish().await;
    }

    Ok(())
}
