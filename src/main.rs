use std::net::TcpListener;

use reliost::configuration::get_configuration;
use reliost::logging::{get_subscriber, init_subscriber};
use reliost::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("reliost".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let settings = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", settings.server.port);
    run(TcpListener::bind(address)?, settings)?.await
}
