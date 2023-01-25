use std::net::TcpListener;

use reliost::{run, get_configuration};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let settings = get_configuration().expect("Failed to read configuration");
    let address = format!("127.0.0.1:{}", settings.server.port);
    run(TcpListener::bind(address)?, settings)?.await
}
