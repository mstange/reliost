//! Build script for the Reliost version.json file.
//!
//! Generates a version.json embedded into the binary via include_str!, served
//! by the /__version__ endpoint per the Dockerflow spec:
//! https://github.com/mozilla-services/Dockerflow/blob/main/docs/version_object.md

use std::{env, fs, path::Path, process::Command};

fn main() {
    // Standard GitHub Actions environment variables.
    println!("cargo:rerun-if-env-changed=GITHUB_SERVER_URL");
    println!("cargo:rerun-if-env-changed=GITHUB_REPOSITORY");
    println!("cargo:rerun-if-env-changed=GITHUB_RUN_ID");
    // Set by cargo from Cargo.toml.
    println!("cargo:rerun-if-env-changed=CARGO_PKG_REPOSITORY");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

    generate_version_file();
}

fn generate_version_file() {
    let repo_url = env!("CARGO_PKG_REPOSITORY");
    let version = env!("CARGO_PKG_VERSION");

    // Construct a link to the CI run from standard GitHub Actions env vars.
    let build_url = match (
        option_env!("GITHUB_SERVER_URL"),
        option_env!("GITHUB_REPOSITORY"),
        option_env!("GITHUB_RUN_ID"),
    ) {
        (Some(server), Some(repo), Some(run_id)) => {
            format!("{}/{}/actions/runs/{}", server, repo, run_id)
        }
        _ => String::new(),
    };

    // Get the commit hash of HEAD.
    let commit_hash_buf = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");
    let commit_hash = std::str::from_utf8(&commit_hash_buf.stdout)
        .expect("Invalid UTF-8 in git output")
        .trim();

    // JSON per the Dockerflow spec.
    let file_output = format!(
        r#"{{ "source": "{}", "version": "{}", "commit": "{}", "build": "{}" }}"#,
        repo_url, version, commit_hash, build_url
    );

    // Written to OUT_DIR so it can be embedded via include_str! in
    // src/routes/dockerflow.rs.
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let file_path = Path::new(&out_dir).join("version.json");
    fs::write(file_path, file_output).expect("Unable to write file");
}
