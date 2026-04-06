use std::path::PathBuf;

use actix_web::{web, HttpRequest, HttpResponse};

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char);
            }
            b => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub async fn self_profiles_index(
    req: HttpRequest,
    dir: web::Data<Option<PathBuf>>,
) -> HttpResponse {
    let Some(dir) = dir.as_ref() else {
        return HttpResponse::NotFound().finish();
    };
    if !dir.join("latest.json.gz").exists() {
        return HttpResponse::ServiceUnavailable().body("No profile recorded yet");
    }
    let conn = req.connection_info();
    let profile_url = format!(
        "{}://{}/self-profiles/latest.json.gz",
        conn.scheme(),
        conn.host()
    );
    let profiler_url = format!(
        "https://profiler.firefox.com/from-url/{}/",
        percent_encode(&profile_url)
    );
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Self Profiles</title></head>
<body>
<h1>Self Profiles</h1>
<p><a href="{profiler_url}">Open latest profile in Firefox Profiler</a></p>
<p><a href="latest.json.gz">Download latest.json.gz</a></p>
</body>
</html>
"#
    );
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub async fn self_profiles_latest(dir: web::Data<Option<PathBuf>>) -> HttpResponse {
    let Some(dir) = dir.as_ref() else {
        return HttpResponse::NotFound().finish();
    };
    match tokio::fs::read(dir.join("latest.json.gz")).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json")
            .insert_header(("Content-Encoding", "gzip"))
            .body(data),
        Err(_) => HttpResponse::NotFound().finish(),
    }
}
