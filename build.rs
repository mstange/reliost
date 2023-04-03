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

    // It's not worth to add serde and serde_json build-dependency just for this.
    // This is simple enough and works just fine.
    let file_output = format!(
        r#" {{ "source": "{}", "version": "{}", "commit": "{}", "build": "{}" }}"#,
        repo_url, version, commit_hash, build_url
    );

    // Use OUT_DIR for debug builds to not clutter the root directory. Output to
    // the root directory if it's a release build so we can find and copy it quickly.
    // This is a workaround until `--out-dir` stabilizes. See:
    // https://github.com/rust-lang/cargo/issues/6790
    let profile =
        std::env::var("PROFILE").expect("Failed to find the PROFILE that is set by Cargo");
    // PROFILE could be either debug or release.
    let out_dir = match &*profile {
        "release" => ".".to_string(),
        _ => env::var("OUT_DIR").expect("Failed to find the OUT_DIR"),
    };
    let file_path = Path::new(&out_dir).join("version.json");

    // Let's output the version.json file.
    fs::write(file_path, file_output).expect("Unable to write file");
}
