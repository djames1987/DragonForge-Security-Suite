#![forbid(unsafe_code)]

use std::path::PathBuf;

use dragonforge_secure_share::{
    CreateShareOptions, ShareSummary, create_share, extract_attachments, reveal_secret,
    verify_share,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
struct SecureShareInfo {
    name: &'static str,
    format_extension: &'static str,
    offline_revocation: bool,
    open_count_enforcement: bool,
    stores_passwords: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct CreateShareRequest {
    sender_label: String,
    recipient_label: String,
    expires_at_ms: u64,
    secret_text: Option<String>,
    attachment_paths: Vec<String>,
    destination: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SharePasswordRequest {
    share_path: String,
    password: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ExtractRequest {
    share_path: String,
    destination: String,
    password: String,
}

#[derive(Debug, Clone, Serialize)]
struct RevealedSecret {
    summary: ShareSummary,
    secret_text: Option<String>,
}

#[tauri::command]
fn secure_share_info() -> SecureShareInfo {
    SecureShareInfo {
        name: "DragonForge Secure Share",
        format_extension: ".dfshare",
        offline_revocation: false,
        open_count_enforcement: false,
        stores_passwords: false,
    }
}

#[tauri::command]
fn create_secure_share(request: CreateShareRequest) -> Result<ShareSummary, String> {
    let attachment_paths = request
        .attachment_paths
        .iter()
        .map(|value| validated_path(value))
        .collect::<Result<Vec<_>, _>>()?;
    let destination = validated_path(&request.destination)?;
    let options = CreateShareOptions {
        sender_label: request.sender_label,
        recipient_label: request.recipient_label,
        expires_at_ms: request.expires_at_ms,
        secret_text: request.secret_text,
        attachment_paths,
    };
    create_share(&options, &destination, &request.password).map_err(|error| error.to_string())
}

#[tauri::command]
fn verify_secure_share(request: SharePasswordRequest) -> Result<ShareSummary, String> {
    let path = validated_path(&request.share_path)?;
    verify_share(&path, &request.password).map_err(|error| error.to_string())
}

#[tauri::command]
fn reveal_secure_share_secret(request: SharePasswordRequest) -> Result<RevealedSecret, String> {
    let path = validated_path(&request.share_path)?;
    let summary = verify_share(&path, &request.password).map_err(|error| error.to_string())?;
    let secret_text = reveal_secret(&path, &request.password).map_err(|error| error.to_string())?;
    Ok(RevealedSecret {
        summary,
        secret_text,
    })
}

#[tauri::command]
fn extract_secure_share_attachments(request: ExtractRequest) -> Result<ShareSummary, String> {
    let path = validated_path(&request.share_path)?;
    let destination = validated_path(&request.destination)?;
    extract_attachments(&path, &request.password, &destination).map_err(|error| error.to_string())
}

fn validated_path(value: &str) -> Result<PathBuf, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err("A required local path is empty.".to_owned());
    }
    if trimmed.chars().any(char::is_control) {
        return Err("A local path contains unsupported control characters.".to_owned());
    }
    Ok(PathBuf::from(trimmed))
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            secure_share_info,
            create_secure_share,
            verify_secure_share,
            reveal_secure_share_secret,
            extract_secure_share_attachments
        ])
        .run(tauri::generate_context!())
        .expect("error while running DragonForge Secure Share");
}

#[cfg(test)]
mod tests {
    use super::{secure_share_info, validated_path};

    #[test]
    fn phase10_does_not_claim_offline_revocation_or_open_counts() {
        let info = secure_share_info();
        assert!(!info.offline_revocation);
        assert!(!info.open_count_enforcement);
        assert!(!info.stores_passwords);
    }

    #[test]
    fn empty_paths_are_rejected() {
        assert!(validated_path("   ").is_err());
    }
}
