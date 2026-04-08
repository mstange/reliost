#[tokio::test]
async fn heartbeat_works() {
    let (address, _join_handle) = super::spawn_app();

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
    let (address, _join_handle) = super::spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{address}/__lbheartbeat__"))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

#[tokio::test]
async fn version_returns_json() {
    let (address, _join_handle) = super::spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{address}/__version__"))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("application/json"),
        "expected application/json, got {content_type:?}"
    );

    let body: serde_json::Value = response.json().await.expect("Response was not valid JSON");
    assert!(body.get("source").is_some(), "missing 'source' field");
    assert!(body.get("version").is_some(), "missing 'version' field");
    assert!(body.get("commit").is_some(), "missing 'commit' field");
    assert!(body.get("build").is_some(), "missing 'build' field");
}
