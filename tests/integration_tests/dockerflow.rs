use std::net::TcpListener;

use reliost::{configuration::ServerSettings, configuration::Settings};
use tokio::task::JoinHandle;

#[tokio::test]
async fn heartbeat_works() {
    let (address, _join_handle) = spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{address}/__heartbeat__"))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

#[tokio::test]
async fn lbheartbeat_works() {
    let (address, _join_handle) = spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{address}/__lbheartbeat__"))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

fn spawn_app() -> (String, JoinHandle<Result<(), std::io::Error>>) {
    let host = "127.0.0.1";
    let listener = TcpListener::bind(format!("{host}:0")).expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let settings = Settings {
        server: ServerSettings {
            host: host.to_string(),
            port,
        },
        symbols: None,
    };
    let server = reliost::startup::run(listener, settings).expect("Failed to bind address.");
    let join_handle = tokio::spawn(server);
    (format!("{host}:{port}"), join_handle)
}
