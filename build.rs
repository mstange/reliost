//! Build script for the Reliost version.json file.
//!
//! This script generates the version file, as requested by the Dockerflow
//! requirements described in
//! https://github.com/mozilla-services/Dockerflow/blob/main/docs/version_object.md
//! It also uses the environment variables set by Github Actions if available.

use std::{env, fs, path::Path, process::Command};

fn main() {
    // These two environment variables are provided by Github Actions.
    // If they are not provided, a dev version of this file will be generated.
    println!("cargo:rerun-if-env-changed=GITHUB_BUILD_URL");
    println!("cargo:rerun-if-env-changed=GITHUB_RUN_ID");
    // These two environment variables are being set by cargo using Cargo.toml.
    println!("cargo:rerun-if-env-changed=CARGO_PKG_REPOSITORY");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    // This environment variable is set by cargo.
    println!("cargo:rerun-if-env-changed=CARGO_MANIFEST_DIR");

    generate_version_file();
}

fn generate_version_file() {
    // Get the repo information from Cargo.toml file.
    let repo_url = env!("CARGO_PKG_REPOSITORY");
    // Get the build url. It's set by Github Actions.
    let build_url = option_env!("GITHUB_BUILD_URL").unwrap_or("");
    // Get the Github Actions run id. This is also set by Github Actions.
    let github_run_id = option_env!("GITHUB_RUN_ID");
    // Convert the run id into version.
    let version = match github_run_id {
        Some(run_id) => format!("0.0.{}", run_id),
        None => env!("CARGO_PKG_VERSION").to_string(),
    };

    // Get the commit hash of HEAD.
    let commit_hash_buf = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");
    let commit_hash = match std::str::from_utf8(&commit_hash_buf.stdout) {
        Ok(hash) => hash.trim(),
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    // Create JSON of the shape required by Mozilla's Dockerflow specification:
    // https://github.com/mozilla-services/Dockerflow/blob/main/docs/version_object.md
    let file_output = format!(
        r#"{{ "source": "{}", "version": "{}", "commit": "{}", "build": "{}" }}"#,
        repo_url, version, commit_hash, build_url
    );

    // We place version.json into the package root, next to Cargo.toml.
    let file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("version.json");

    // Output the version.json file.
    fs::write(file_path, file_output).expect("Unable to write file");
}
