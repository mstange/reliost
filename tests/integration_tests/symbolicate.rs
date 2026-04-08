// Minimal valid v5 request: one job with no modules and one empty stack.
// No symbol server is needed — we just want to verify the HTTP contract.
const EMPTY_JOB_REQUEST: &str = r#"{"memoryMap":[],"stacks":[[]]}"#;

#[tokio::test]
async fn symbolicate_v5_returns_200_with_json_content_type() {
    let (address, _join_handle) = super::spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{address}/symbolicate/v5"))
        .header("Content-Type", "application/json")
        .body(EMPTY_JOB_REQUEST)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), 200);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.contains("application/json"),
        "expected application/json, got {content_type:?}"
    );
}

#[tokio::test]
async fn symbolicate_v5_response_has_results_field() {
    let (address, _join_handle) = super::spawn_app();

    // reqwest decompresses gzip by default, so we can read the JSON directly.
    let client = reqwest::Client::new();
    let body: serde_json::Value = client
        .post(format!("http://{address}/symbolicate/v5"))
        .header("Content-Type", "application/json")
        .body(EMPTY_JOB_REQUEST)
        .send()
        .await
        .expect("Failed to execute request.")
        .json()
        .await
        .expect("Response was not valid JSON");

    let results = body
        .get("results")
        .and_then(|v| v.as_array())
        .expect("response should have a 'results' array");

    // One job in → one result out.
    assert_eq!(results.len(), 1);

    let result = &results[0];
    assert!(
        result.get("stacks").is_some(),
        "result should have a 'stacks' field"
    );
    assert!(
        result.get("found_modules").is_some(),
        "result should have a 'found_modules' field"
    );
}

#[tokio::test]
async fn symbolicate_v5_multi_job_request() {
    let (address, _join_handle) = super::spawn_app();

    // Two jobs, each with an empty stack.
    let request = r#"{"jobs":[{"memoryMap":[],"stacks":[[]]},{"memoryMap":[],"stacks":[[]]}]}"#;

    let client = reqwest::Client::new();
    let body: serde_json::Value = client
        .post(format!("http://{address}/symbolicate/v5"))
        .header("Content-Type", "application/json")
        .body(request)
        .send()
        .await
        .expect("Failed to execute request.")
        .json()
        .await
        .expect("Response was not valid JSON");

    let results = body
        .get("results")
        .and_then(|v| v.as_array())
        .expect("response should have a 'results' array");

    assert_eq!(results.len(), 2, "two jobs in should produce two results");
}

#[tokio::test]
async fn symbolicate_v5_cors_header_on_response() {
    let (address, _join_handle) = super::spawn_app();

    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{address}/symbolicate/v5"))
        .header("Content-Type", "application/json")
        .header("Origin", "https://profiler.firefox.com")
        .body(EMPTY_JOB_REQUEST)
        .send()
        .await
        .expect("Failed to execute request.");

    let acao = response
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(
        acao, "*",
        "expected Access-Control-Allow-Origin: *, got {acao:?}"
    );
}

#[tokio::test]
async fn symbolicate_v5_response_is_gzip_encoded() {
    let (address, _join_handle) = super::spawn_app();

    // Disable automatic decompression so we can inspect the raw Content-Encoding header.
    let client = reqwest::Client::builder()
        .no_gzip()
        .build()
        .expect("Failed to build client.");

    let response = client
        .post(format!("http://{address}/symbolicate/v5"))
        .header("Content-Type", "application/json")
        .body(EMPTY_JOB_REQUEST)
        .send()
        .await
        .expect("Failed to execute request.");

    let encoding = response
        .headers()
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(
        encoding, "gzip",
        "expected Content-Encoding: gzip, got {encoding:?}"
    );
}
