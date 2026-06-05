use std::{env, fs, path::PathBuf};

fn main() {
    let output_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("openapi.json"));

    let parent = output_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map(PathBuf::from);
    if let Some(parent) = parent {
        fs::create_dir_all(parent).expect("failed to create OpenAPI output directory");
    }

    let document = mail_shell_server::api_docs::openapi_doc()
        .to_pretty_json()
        .expect("failed to serialize OpenAPI document");
    fs::write(&output_path, document).expect("failed to write OpenAPI document");
}
