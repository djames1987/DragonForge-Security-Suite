use std::collections::HashSet;
use std::fs;
use std::path::{Component as PathComponent, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{Result, ShareError};
use crate::format::{
    FORMAT_VERSION, HEADER_LEN, MAGIC, MAX_ATTACHMENTS, MAX_FILE_BYTES, MAX_LABEL_CHARS,
    MAX_PACKAGE_BYTES, MAX_PATH_CHARS, MAX_SECRET_BYTES, MAX_TOTAL_BYTES, NONCE_LEN, SALT_LEN,
    TAG_LEN,
};

const ARGON2_MEMORY_KIB: u32 = 65_536;
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_LANES: u32 = 1;
const MIN_PASSWORD_CHARS: usize = 12;
const MAX_EXPIRATION_WINDOW_MS: u64 = 366 * 24 * 60 * 60 * 1_000;

#[derive(Debug, Clone)]
pub struct CreateShareOptions {
    pub sender_label: String,
    pub recipient_label: String,
    pub expires_at_ms: u64,
    pub secret_text: Option<String>,
    pub attachment_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShareSummary {
    pub format_version: u16,
    pub created_at_ms: u64,
    pub expires_at_ms: u64,
    pub expired: bool,
    pub sender_label: String,
    pub recipient_label: String,
    pub has_secret: bool,
    pub attachment_count: usize,
    pub total_attachment_bytes: u64,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharePayload {
    format_version: u16,
    created_at_ms: u64,
    expires_at_ms: u64,
    sender_label: String,
    recipient_label: String,
    secret_text: Option<String>,
    attachments: Vec<StoredAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredAttachment {
    archive_path: String,
    sha256: String,
    size: u64,
    data_b64: String,
}

pub fn create_share(
    options: &CreateShareOptions,
    destination: &Path,
    password: &str,
) -> Result<ShareSummary> {
    validate_password(password)?;
    validate_options(options)?;
    if destination.exists() {
        return Err(ShareError::Conflict("share destination already exists"));
    }

    let created_at_ms = now_ms();
    if options.expires_at_ms <= created_at_ms {
        return Err(ShareError::InvalidInput(
            "share expiration must be in the future",
        ));
    }
    if options.expires_at_ms - created_at_ms > MAX_EXPIRATION_WINDOW_MS {
        return Err(ShareError::InvalidInput(
            "share expiration cannot exceed 366 days",
        ));
    }

    let mut attachments = Vec::new();
    let mut total_bytes = 0_u64;
    for (index, source) in options.attachment_paths.iter().enumerate() {
        collect_source(index, source, &mut attachments, &mut total_bytes)?;
    }

    let payload = SharePayload {
        format_version: FORMAT_VERSION,
        created_at_ms,
        expires_at_ms: options.expires_at_ms,
        sender_label: options.sender_label.trim().to_owned(),
        recipient_label: options.recipient_label.trim().to_owned(),
        secret_text: options.secret_text.clone(),
        attachments,
    };
    validate_payload(&payload)?;

    let plaintext = serde_json::to_vec(&payload)
        .map_err(|_| ShareError::Format("share payload could not be serialized"))?;
    if plaintext.len() as u64 > MAX_PACKAGE_BYTES {
        return Err(ShareError::InvalidInput(
            "share payload exceeds the safe package limit",
        ));
    }

    let mut salt = [0_u8; SALT_LEN];
    let mut nonce_bytes = [0_u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key[..])
        .map_err(|_| ShareError::Crypto("share cipher could not be initialized"))?;
    let cipher_len = plaintext
        .len()
        .checked_add(TAG_LEN)
        .ok_or(ShareError::InvalidInput("share payload is too large"))?
        as u64;
    let aad = build_header(&salt, &nonce_bytes, cipher_len);
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: &plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| ShareError::Crypto("share encryption failed"))?;

    if let Some(parent) = nonempty_parent(destination) {
        fs::create_dir_all(parent)
            .map_err(|_| ShareError::Io("share destination directory could not be created"))?;
    }
    let temporary = temporary_sibling(destination)?;
    fs::write(&temporary, [&aad[..], &ciphertext].concat())
        .map_err(|_| ShareError::Io("encrypted share could not be written"))?;
    if fs::rename(&temporary, destination).is_err() {
        let _ = fs::remove_file(&temporary);
        return Err(ShareError::Io("encrypted share could not be finalized"));
    }

    verify_share(destination, password)
}

pub fn verify_share(path: &Path, password: &str) -> Result<ShareSummary> {
    let payload = decrypt_payload(path, password)?;
    validate_payload(&payload)?;
    summary_from_payload(&payload, true)
}

pub fn reveal_secret(path: &Path, password: &str) -> Result<Option<String>> {
    let payload = decrypt_payload(path, password)?;
    validate_payload(&payload)?;
    reject_if_expired(&payload)?;
    Ok(payload.secret_text)
}

pub fn extract_attachments(
    path: &Path,
    password: &str,
    destination: &Path,
) -> Result<ShareSummary> {
    if destination.exists() {
        return Err(ShareError::Conflict(
            "attachment extraction destination already exists",
        ));
    }

    let payload = decrypt_payload(path, password)?;
    validate_payload(&payload)?;
    reject_if_expired(&payload)?;

    if let Some(parent) = nonempty_parent(destination) {
        fs::create_dir_all(parent)
            .map_err(|_| ShareError::Io("extraction parent directory could not be created"))?;
    }
    let temporary = temporary_sibling(destination)?;
    fs::create_dir(&temporary)
        .map_err(|_| ShareError::Io("temporary extraction directory could not be created"))?;

    let result = (|| {
        for attachment in &payload.attachments {
            let relative = validated_relative_path(&attachment.archive_path)?;
            let target = temporary.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|_| ShareError::Io("attachment directory could not be created"))?;
            }
            let bytes = decoded_verified_bytes(attachment)?;
            fs::write(&target, bytes)
                .map_err(|_| ShareError::Io("attachment could not be written"))?;
        }
        fs::rename(&temporary, destination)
            .map_err(|_| ShareError::Io("attachment extraction could not be finalized"))?;
        summary_from_payload(&payload, true)
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}

fn collect_source(
    index: usize,
    source: &Path,
    attachments: &mut Vec<StoredAttachment>,
    total_bytes: &mut u64,
) -> Result<()> {
    let metadata = fs::symlink_metadata(source)
        .map_err(|_| ShareError::Io("an attachment source could not be read"))?;
    if metadata.file_type().is_symlink() {
        return Err(ShareError::InvalidInput(
            "symbolic-link attachment sources are not allowed",
        ));
    }
    let basename = source
        .file_name()
        .and_then(|value| value.to_str())
        .map(sanitize_segment)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "attachment".to_owned());
    let root = format!("attachment-{:02}-{}", index + 1, basename);

    if metadata.is_file() {
        collect_file(source, &root, attachments, total_bytes)
    } else if metadata.is_dir() {
        collect_directory(source, Path::new(&root), attachments, total_bytes)
    } else {
        Err(ShareError::InvalidInput(
            "attachments must be regular files or directories",
        ))
    }
}

fn collect_directory(
    directory: &Path,
    archive_root: &Path,
    attachments: &mut Vec<StoredAttachment>,
    total_bytes: &mut u64,
) -> Result<()> {
    let mut children = fs::read_dir(directory)
        .map_err(|_| ShareError::Io("attachment directory could not be read"))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| ShareError::Io("attachment directory entries could not be read"))?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| ShareError::Io("attachment metadata could not be read"))?;
        if metadata.file_type().is_symlink() {
            return Err(ShareError::InvalidInput(
                "symbolic links inside attachment folders are not allowed",
            ));
        }
        let name = child
            .file_name()
            .to_str()
            .map(sanitize_segment)
            .filter(|value| !value.is_empty())
            .ok_or(ShareError::InvalidInput(
                "attachment contains an unsupported file name",
            ))?;
        let archive_path = archive_root.join(name);
        if metadata.is_dir() {
            collect_directory(&path, &archive_path, attachments, total_bytes)?;
        } else if metadata.is_file() {
            let archive = archive_path.to_str().ok_or(ShareError::InvalidInput(
                "attachment path could not be represented safely",
            ))?;
            collect_file(&path, archive, attachments, total_bytes)?;
        }
    }
    Ok(())
}

fn collect_file(
    source: &Path,
    archive_path: &str,
    attachments: &mut Vec<StoredAttachment>,
    total_bytes: &mut u64,
) -> Result<()> {
    if attachments.len() >= MAX_ATTACHMENTS {
        return Err(ShareError::InvalidInput(
            "share contains too many attachment files",
        ));
    }
    if archive_path.chars().count() > MAX_PATH_CHARS {
        return Err(ShareError::InvalidInput(
            "attachment path exceeds the safe length limit",
        ));
    }

    let metadata = fs::metadata(source)
        .map_err(|_| ShareError::Io("attachment metadata could not be read"))?;
    if metadata.len() > MAX_FILE_BYTES {
        return Err(ShareError::InvalidInput(
            "an individual attachment exceeds the safe size limit",
        ));
    }
    let next_total = total_bytes
        .checked_add(metadata.len())
        .ok_or(ShareError::InvalidInput("attachment size overflowed"))?;
    if next_total > MAX_TOTAL_BYTES {
        return Err(ShareError::InvalidInput(
            "share exceeds the safe total attachment-size limit",
        ));
    }

    let bytes = fs::read(source).map_err(|_| ShareError::Io("attachment could not be read"))?;
    let normalized = archive_path.replace('\\', "/");
    validated_relative_path(&normalized)?;
    if attachments
        .iter()
        .any(|attachment| attachment.archive_path == normalized)
    {
        return Err(ShareError::InvalidInput(
            "attachment sources produce duplicate archive paths",
        ));
    }

    attachments.push(StoredAttachment {
        archive_path: normalized,
        sha256: digest_hex(&bytes),
        size: bytes.len() as u64,
        data_b64: BASE64.encode(bytes),
    });
    *total_bytes = next_total;
    Ok(())
}

fn decrypt_payload(path: &Path, password: &str) -> Result<SharePayload> {
    validate_password(password)?;
    let metadata =
        fs::metadata(path).map_err(|_| ShareError::Io("share file could not be read"))?;
    if metadata.len() > MAX_PACKAGE_BYTES + HEADER_LEN as u64 {
        return Err(ShareError::Format(
            "share file exceeds the safe package limit",
        ));
    }

    let bytes = fs::read(path).map_err(|_| ShareError::Io("share file could not be read"))?;
    if bytes.len() < HEADER_LEN + TAG_LEN {
        return Err(ShareError::Format("share file is truncated"));
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        return Err(ShareError::Format("share file magic is invalid"));
    }

    let version_offset = MAGIC.len();
    let version = u16::from_le_bytes([bytes[version_offset], bytes[version_offset + 1]]);
    if version != FORMAT_VERSION {
        return Err(ShareError::Format("share format version is unsupported"));
    }

    let salt_start = version_offset + 2;
    let nonce_start = salt_start + SALT_LEN;
    let length_start = nonce_start + NONCE_LEN;
    let salt: [u8; SALT_LEN] = bytes[salt_start..nonce_start]
        .try_into()
        .map_err(|_| ShareError::Format("share salt is malformed"))?;
    let nonce: [u8; NONCE_LEN] = bytes[nonce_start..length_start]
        .try_into()
        .map_err(|_| ShareError::Format("share nonce is malformed"))?;
    let cipher_len = u64::from_le_bytes(
        bytes[length_start..HEADER_LEN]
            .try_into()
            .map_err(|_| ShareError::Format("share length is malformed"))?,
    );
    if cipher_len as usize != bytes.len() - HEADER_LEN {
        return Err(ShareError::Format("share ciphertext length is invalid"));
    }

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key[..])
        .map_err(|_| ShareError::Crypto("share cipher could not be initialized"))?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &bytes[HEADER_LEN..],
                aad: &bytes[..HEADER_LEN],
            },
        )
        .map_err(|_| {
            ShareError::Crypto("share password is incorrect or the package was modified")
        })?;

    let payload: SharePayload = serde_json::from_slice(&plaintext)
        .map_err(|_| ShareError::Format("decrypted share payload is malformed"))?;
    if payload.format_version != FORMAT_VERSION {
        return Err(ShareError::Format(
            "decrypted share format version is unsupported",
        ));
    }
    Ok(payload)
}

fn validate_options(options: &CreateShareOptions) -> Result<()> {
    validate_label(&options.recipient_label, true)?;
    validate_label(&options.sender_label, false)?;
    if let Some(secret) = &options.secret_text {
        if secret.as_bytes().len() > MAX_SECRET_BYTES {
            return Err(ShareError::InvalidInput(
                "secret text exceeds the safe size limit",
            ));
        }
    }
    if options
        .secret_text
        .as_ref()
        .is_none_or(|secret| secret.is_empty())
        && options.attachment_paths.is_empty()
    {
        return Err(ShareError::InvalidInput(
            "share must contain secret text or at least one attachment",
        ));
    }
    if options.attachment_paths.len() > 64 {
        return Err(ShareError::InvalidInput(
            "too many top-level attachment sources were requested",
        ));
    }
    Ok(())
}

fn validate_payload(payload: &SharePayload) -> Result<()> {
    validate_label(&payload.recipient_label, true)?;
    validate_label(&payload.sender_label, false)?;
    if payload.expires_at_ms <= payload.created_at_ms {
        return Err(ShareError::Integrity(
            "share expiration is not after its creation time",
        ));
    }
    if payload.expires_at_ms - payload.created_at_ms > MAX_EXPIRATION_WINDOW_MS {
        return Err(ShareError::Integrity(
            "share expiration window exceeds the supported limit",
        ));
    }
    if let Some(secret) = &payload.secret_text {
        if secret.as_bytes().len() > MAX_SECRET_BYTES {
            return Err(ShareError::Integrity(
                "share secret exceeds the safe size limit",
            ));
        }
    }
    if payload
        .secret_text
        .as_ref()
        .is_none_or(|secret| secret.is_empty())
        && payload.attachments.is_empty()
    {
        return Err(ShareError::Integrity("share contains no protected content"));
    }
    if payload.attachments.len() > MAX_ATTACHMENTS {
        return Err(ShareError::Integrity(
            "share contains too many attachment files",
        ));
    }

    let mut seen = HashSet::new();
    let mut total = 0_u64;
    for attachment in &payload.attachments {
        if !seen.insert(attachment.archive_path.clone()) {
            return Err(ShareError::Integrity(
                "share contains duplicate attachment paths",
            ));
        }
        validated_relative_path(&attachment.archive_path)?;
        if attachment.size > MAX_FILE_BYTES {
            return Err(ShareError::Integrity(
                "share attachment exceeds the safe file-size limit",
            ));
        }
        let _ = decoded_verified_bytes(attachment)?;
        total = total
            .checked_add(attachment.size)
            .ok_or(ShareError::Integrity("share attachment byte count overflowed"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(ShareError::Integrity(
                "share exceeds the safe total attachment-size limit",
            ));
        }
    }
    Ok(())
}

fn decoded_verified_bytes(attachment: &StoredAttachment) -> Result<Vec<u8>> {
    let bytes = BASE64
        .decode(&attachment.data_b64)
        .map_err(|_| ShareError::Format("attachment data is malformed"))?;
    if bytes.len() as u64 != attachment.size || digest_hex(&bytes) != attachment.sha256 {
        return Err(ShareError::Integrity(
            "share attachment integrity verification failed",
        ));
    }
    Ok(bytes)
}

fn summary_from_payload(payload: &SharePayload, verified: bool) -> Result<ShareSummary> {
    let total_attachment_bytes = payload.attachments.iter().try_fold(0_u64, |total, item| {
        total
            .checked_add(item.size)
            .ok_or(ShareError::Integrity("share attachment byte count overflowed"))
    })?;
    Ok(ShareSummary {
        format_version: payload.format_version,
        created_at_ms: payload.created_at_ms,
        expires_at_ms: payload.expires_at_ms,
        expired: now_ms() >= payload.expires_at_ms,
        sender_label: payload.sender_label.clone(),
        recipient_label: payload.recipient_label.clone(),
        has_secret: payload.secret_text.as_ref().is_some_and(|secret| !secret.is_empty()),
        attachment_count: payload.attachments.len(),
        total_attachment_bytes,
        verified,
    })
}

fn reject_if_expired(payload: &SharePayload) -> Result<()> {
    if now_ms() >= payload.expires_at_ms {
        return Err(ShareError::Expired("secure share has expired"));
    }
    Ok(())
}

fn validate_label(value: &str, required: bool) -> Result<()> {
    let trimmed = value.trim();
    if required && trimmed.is_empty() {
        return Err(ShareError::InvalidInput("recipient label is required"));
    }
    if trimmed.chars().count() > MAX_LABEL_CHARS || trimmed.chars().any(char::is_control) {
        return Err(ShareError::InvalidInput("share label is invalid"));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<()> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(ShareError::InvalidInput(
            "share password must contain at least 12 characters",
        ));
    }
    Ok(())
}

fn derive_key(password: &str, salt: &[u8; SALT_LEN]) -> Result<Zeroizing<[u8; 32]>> {
    let params = Params::new(ARGON2_MEMORY_KIB, ARGON2_ITERATIONS, ARGON2_LANES, Some(32))
        .map_err(|_| ShareError::Crypto("share key-derivation parameters are invalid"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0_u8; 32]);
    argon2
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|_| ShareError::Crypto("share key derivation failed"))?;
    Ok(key)
}

fn build_header(salt: &[u8; SALT_LEN], nonce: &[u8; NONCE_LEN], cipher_len: u64) -> Vec<u8> {
    let mut header = Vec::with_capacity(HEADER_LEN);
    header.extend_from_slice(MAGIC);
    header.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    header.extend_from_slice(salt);
    header.extend_from_slice(nonce);
    header.extend_from_slice(&cipher_len.to_le_bytes());
    header
}

fn validated_relative_path(value: &str) -> Result<PathBuf> {
    if value.is_empty() || value.chars().count() > MAX_PATH_CHARS {
        return Err(ShareError::Integrity(
            "share contains an invalid attachment path",
        ));
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(ShareError::Integrity(
            "share contains an absolute attachment path",
        ));
    }
    for component in path.components() {
        if !matches!(component, PathComponent::Normal(_)) {
            return Err(ShareError::Integrity(
                "share contains a traversal-like attachment path",
            ));
        }
    }
    Ok(path.to_path_buf())
}

fn sanitize_segment(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            !character.is_control()
                && !matches!(
                    character,
                    '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
                )
        })
        .take(120)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn digest_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn nonempty_parent(path: &Path) -> Option<&Path> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
}

fn temporary_sibling(destination: &Path) -> Result<PathBuf> {
    let parent = nonempty_parent(destination).unwrap_or_else(|| Path::new("."));
    let mut random = [0_u8; 8];
    OsRng.fill_bytes(&mut random);
    let token = random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("dragonforge-share");
    Ok(parent.join(format!(".{name}.{token}.tmp")))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().min(u128::from(u64::MAX)) as u64
        })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use tempfile::tempdir;

    use super::{
        CreateShareOptions, create_share, extract_attachments, reveal_secret, verify_share,
    };

    const PASSWORD: &str = "correct horse battery staple";

    fn future_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_millis() as u64
            + 60_000
    }

    fn basic_options() -> CreateShareOptions {
        CreateShareOptions {
            sender_label: "Alice".to_owned(),
            recipient_label: "Bob".to_owned(),
            expires_at_ms: future_ms(),
            secret_text: Some("launch code 1234".to_owned()),
            attachment_paths: Vec::new(),
        }
    }

    #[test]
    fn encrypted_secret_round_trip() {
        let dir = tempdir().expect("tempdir");
        let package = dir.path().join("secret.dfshare");
        let summary = create_share(&basic_options(), &package, PASSWORD).expect("create");
        assert!(summary.verified);
        assert_eq!(summary.recipient_label, "Bob");
        assert_eq!(
            reveal_secret(&package, PASSWORD).expect("reveal"),
            Some("launch code 1234".to_owned())
        );
    }

    #[test]
    fn attachment_round_trip_extracts_without_overwrite() {
        let dir = tempdir().expect("tempdir");
        let source = dir.path().join("note.txt");
        fs::write(&source, b"hello").expect("source");
        let mut options = basic_options();
        options.secret_text = None;
        options.attachment_paths = vec![source];

        let package = dir.path().join("files.dfshare");
        create_share(&options, &package, PASSWORD).expect("create");
        let output = dir.path().join("output");
        let summary = extract_attachments(&package, PASSWORD, &output).expect("extract");
        assert_eq!(summary.attachment_count, 1);
        assert!(extract_attachments(&package, PASSWORD, &output).is_err());
    }

    #[test]
    fn wrong_password_and_tampering_are_rejected() {
        let dir = tempdir().expect("tempdir");
        let package = dir.path().join("secret.dfshare");
        create_share(&basic_options(), &package, PASSWORD).expect("create");
        assert!(verify_share(&package, "this password is wrong").is_err());

        let mut bytes = fs::read(&package).expect("read");
        let last = bytes.len() - 1;
        bytes[last] ^= 0x42;
        fs::write(&package, bytes).expect("tamper");
        assert!(verify_share(&package, PASSWORD).is_err());
    }

    #[test]
    fn expired_share_can_verify_but_cannot_reveal() {
        let dir = tempdir().expect("tempdir");
        let package = dir.path().join("secret.dfshare");
        create_share(&basic_options(), &package, PASSWORD).expect("create");

        let mut bytes = fs::read(&package).expect("read");
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;
        fs::write(&package, bytes).expect("tamper");
        assert!(verify_share(&package, PASSWORD).is_err());
    }

    #[test]
    fn short_password_is_rejected() {
        let dir = tempdir().expect("tempdir");
        let package = dir.path().join("secret.dfshare");
        assert!(create_share(&basic_options(), &package, "too-short").is_err());
    }

    #[test]
    fn recipient_label_is_required() {
        let dir = tempdir().expect("tempdir");
        let package = dir.path().join("secret.dfshare");
        let mut options = basic_options();
        options.recipient_label.clear();
        assert!(create_share(&options, &package, PASSWORD).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_attachments_are_rejected() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().expect("tempdir");
        let real = dir.path().join("real.txt");
        fs::write(&real, b"secret").expect("real");
        let link = dir.path().join("link.txt");
        symlink(&real, &link).expect("link");
        let mut options = basic_options();
        options.attachment_paths = vec![link];

        let package = dir.path().join("secret.dfshare");
        assert!(create_share(&options, &package, PASSWORD).is_err());
    }
}
