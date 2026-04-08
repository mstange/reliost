#[tokio::test]
async fn unknown_route_returns_404() {
    let (address, _join_handle) = super::spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{address}/does-not-exist"))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(response.status(), 404);
}
