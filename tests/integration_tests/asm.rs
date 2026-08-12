#[tokio::test]
async fn asm_v1_rejects_non_utf8_body_without_killing_the_server() {
    let (address, _join_handle) = super::spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{address}/asm/v1"))
        .header("Content-Type", "application/json")
        .body(vec![0xff, 0xfe, 0xfd])
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), 400);

    // The server must still be alive after a bad request.
    let response = client
        .get(format!("http://{address}/__lbheartbeat__"))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn asm_v1_accepts_a_utf8_body() {
    let (address, _join_handle) = super::spawn_app();

    // No symbols are configured, so this request cannot succeed — we only care
    // that it is parsed and answered rather than panicking.
    let request = r#"{"debugName":"nonexistent","debugId":"000000000000000000000000000000000","startAddress":"0x0","size":"0x10"}"#;

    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{address}/asm/v1"))
        .header("Content-Type", "application/json")
        .body(request)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_ne!(response.status(), 400);
    let _: serde_json::Value = response.json().await.expect("Response was not valid JSON");
}
