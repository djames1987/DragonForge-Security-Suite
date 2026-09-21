use std::collections::HashSet;
use std::fs;
use std::path::{Component as PathComponent, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use dragonforge_core::SuitePaths;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{BackupError, Result};
use crate::format::{
    FORMAT_VERSION, HEADER_LEN, MAGIC, MAX_ARCHIVE_BYTES, MAX_ENTRIES, MAX_FILE_BYTES,
    MAX_PATH_CHARS, MAX_TOTAL_BYTES, NONCE_LEN, SALT_LEN, TAG_LEN,
};

const ARGON2_MEMORY_KIB: u32 = 65_536;
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_LANES: u32 = 1;
const MIN_PASSWORD_CHARS: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiscoveredSource {
    pub label: String,
    pub path: String,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackupSelectionSummary {
    pub label: String,
    pub original_path: String,
    pub entry_count: usize,
    pub byte_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BackupSummary {
    pub format_version: u16,
    pub created_at_ms: u64,
    pub selections: Vec<BackupSelectionSummary>,
    pub entries: usize,
    pub total_bytes: u64,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RestoreSummary {
    pub destination: String,
    pub entries: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupPayload {
    format_version: u16,
    created_at_ms: u64,
    selections: Vec<BackupSelectionSummary>,
    entries: Vec<StoredEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredEntry {
    archive_path: String,
    sha256: String,
    size: u64,
    data_b64: String,
}

pub fn discover_suite_sources() -> Result<Vec<DiscoveredSource>> {
    let paths = SuitePaths::discover()
        .map_err(|_| BackupError::Io("DragonForge suite data paths could not be discovered"))?;
    let mut sources = Vec::new();
    let mut seen = HashSet::new();

    for (label, path) in [
        ("DragonForge configuration", paths.config_root()),
        ("DragonForge application data", paths.data_root()),
    ] {
        let normalized = path.to_path_buf();
        if seen.insert(normalized.clone()) {
            sources.push(DiscoveredSource {
                label: label.to_owned(),
                path: normalized.to_string_lossy().into_owned(),
                exists: normalized.exists(),
            });
        }
    }

    Ok(sources)
}

pub fn create_backup(sources: &[PathBuf], destination: &Path, password: &str) -> Result<BackupSummary> {
    validate_password(password)?;
    if sources.is_empty() {
        return Err(BackupError::InvalidInput("at least one backup source is required"));
    }
    if sources.len() > 64 {
        return Err(BackupError::InvalidInput("too many backup sources were requested"));
    }
    if destination.exists() {
        return Err(BackupError::Conflict("backup destination already exists"));
    }

    let mut entries = Vec::new();
    let mut selections = Vec::new();
    let mut total_bytes = 0_u64;

    for (index, source) in sources.iter().enumerate() {
        let metadata = fs::symlink_metadata(source)
            .map_err(|_| BackupError::Io("a backup source could not be read"))?;
        if metadata.file_type().is_symlink() {
            return Err(BackupError::InvalidInput("symbolic-link backup sources are not allowed"));
        }

        let basename = source
            .file_name()
            .and_then(|value| value.to_str())
            .map(sanitize_segment)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "source".to_owned());
        let root = format!("source-{:02}-{}", index + 1, basename);
        let before_entries = entries.len();
        let before_bytes = total_bytes;

        if metadata.is_file() {
            collect_file(source, &root, &mut entries, &mut total_bytes)?;
        } else if metadata.is_dir() {
            collect_directory(source, Path::new(&root), &mut entries, &mut total_bytes)?;
        } else {
            return Err(BackupError::InvalidInput("backup sources must be regular files or directories"));
        }

        selections.push(BackupSelectionSummary {
            label: basename,
            original_path: source.to_string_lossy().into_owned(),
            entry_count: entries.len() - before_entries,
            byte_count: total_bytes - before_bytes,
        });
    }

    let payload = BackupPayload {
        format_version: FORMAT_VERSION,
        created_at_ms: now_ms(),
        selections,
        entries,
    };
    let plaintext = serde_json::to_vec(&payload)
        .map_err(|_| BackupError::Format("backup payload could not be serialized"))?;
    if plaintext.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err(BackupError::InvalidInput("backup payload exceeds the safe archive limit"));
    }

    let mut salt = [0_u8; SALT_LEN];
    let mut nonce_bytes = [0_u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key[..])
        .map_err(|_| BackupError::Crypto("backup cipher could not be initialized"))?;
    let cipher_len = plaintext
        .len()
        .checked_add(TAG_LEN)
        .ok_or(BackupError::InvalidInput("backup payload is too large"))? as u64;
    let aad = build_header(&salt, &nonce_bytes, cipher_len);
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: &plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| BackupError::Crypto("backup encryption failed"))?;

    if let Some(parent) = nonempty_parent(destination) {
        fs::create_dir_all(parent)
            .map_err(|_| BackupError::Io("backup destination directory could not be created"))?;
    }
    let temporary = temporary_sibling(destination)?;
    fs::write(&temporary, [&aad[..], &ciphertext].concat())
        .map_err(|_| BackupError::Io("encrypted backup could not be written"))?;
    if fs::rename(&temporary, destination).is_err() {
        let _ = fs::remove_file(&temporary);
        return Err(BackupError::Io("encrypted backup could not be finalized"));
    }

    summary_from_payload(&payload, true)
}

pub fn inspect_backup(path: &Path, password: &str) -> Result<BackupSummary> {
    let payload = decrypt_payload(path, password)?;
    summary_from_payload(&payload, false)
}

pub fn verify_backup(path: &Path, password: &str) -> Result<BackupSummary> {
    let payload = decrypt_payload(path, password)?;
    validate_payload(&payload)?;
    summary_from_payload(&payload, true)
}

pub fn restore_backup(path: &Path, password: &str, destination: &Path) -> Result<RestoreSummary> {
    if destination.exists() {
        return Err(BackupError::Conflict("restore destination already exists"));
    }

    let payload = decrypt_payload(path, password)?;
    validate_payload(&payload)?;
    if let Some(parent) = nonempty_parent(destination) {
        fs::create_dir_all(parent)
            .map_err(|_| BackupError::Io("restore parent directory could not be created"))?;
    }

    let temporary = temporary_sibling(destination)?;
    fs::create_dir(&temporary)
        .map_err(|_| BackupError::Io("temporary restore directory could not be created"))?;

    let result = (|| {
        let mut total = 0_u64;
        for entry in &payload.entries {
            let relative = validated_relative_path(&entry.archive_path)?;
            let target = temporary.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|_| BackupError::Io("restore directory could not be created"))?;
            }
            let bytes = BASE64
                .decode(&entry.data_b64)
                .map_err(|_| BackupError::Format("backup file data is malformed"))?;
            if bytes.len() as u64 != entry.size || digest_hex(&bytes) != entry.sha256 {
                return Err(BackupError::Integrity("backup entry integrity verification failed"));
            }
            fs::write(&target, &bytes)
                .map_err(|_| BackupError::Io("restored file could not be written"))?;
            total = total
                .checked_add(entry.size)
                .ok_or(BackupError::Integrity("backup byte count overflowed"))?;
        }
        fs::rename(&temporary, destination)
            .map_err(|_| BackupError::Io("restored data could not be finalized"))?;
        Ok(RestoreSummary {
            destination: destination.to_string_lossy().into_owned(),
            entries: payload.entries.len(),
            total_bytes: total,
        })
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&temporary);
    }
    result
}

fn collect_directory(
    directory: &Path,
    archive_root: &Path,
    entries: &mut Vec<StoredEntry>,
    total_bytes: &mut u64,
) -> Result<()> {
    let mut children = fs::read_dir(directory)
        .map_err(|_| BackupError::Io("backup directory could not be read"))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| BackupError::Io("backup directory entries could not be read"))?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| BackupError::Io("backup source metadata could not be read"))?;
        if metadata.file_type().is_symlink() {
            return Err(BackupError::InvalidInput("symbolic links inside backup sources are not allowed"));
        }
        let name = child
            .file_name()
            .to_str()
            .map(sanitize_segment)
            .filter(|value| !value.is_empty())
            .ok_or(BackupError::InvalidInput("backup source contains an unsupported file name"))?;
        let archive_path = archive_root.join(name);
        if metadata.is_dir() {
            collect_directory(&path, &archive_path, entries, total_bytes)?;
        } else if metadata.is_file() {
            let archive = archive_path
                .to_str()
                .ok_or(BackupError::InvalidInput("backup path could not be represented safely"))?;
            collect_file(&path, archive, entries, total_bytes)?;
        }
    }
    Ok(())
}

fn collect_file(
    source: &Path,
    archive_path: &str,
    entries: &mut Vec<StoredEntry>,
    total_bytes: &mut u64,
) -> Result<()> {
    if entries.len() >= MAX_ENTRIES {
        return Err(BackupError::InvalidInput("backup contains too many files"));
    }
    if archive_path.chars().count() > MAX_PATH_CHARS {
        return Err(BackupError::InvalidInput("backup path exceeds the safe length limit"));
    }
    let metadata = fs::metadata(source)
        .map_err(|_| BackupError::Io("backup file metadata could not be read"))?;
    if metadata.len() > MAX_FILE_BYTES {
        return Err(BackupError::InvalidInput("an individual backup file exceeds the safe size limit"));
    }
    let next_total = total_bytes
        .checked_add(metadata.len())
        .ok_or(BackupError::InvalidInput("backup size overflowed"))?;
    if next_total > MAX_TOTAL_BYTES {
        return Err(BackupError::InvalidInput("backup exceeds the safe total file-size limit"));
    }

    let bytes = fs::read(source).map_err(|_| BackupError::Io("backup file could not be read"))?;
    let normalized = archive_path.replace('\\', "/");
    validated_relative_path(&normalized)?;
    if entries.iter().any(|entry| entry.archive_path == normalized) {
        return Err(BackupError::InvalidInput(
            "backup sources produce duplicate archive paths",
        ));
    }
    entries.push(StoredEntry {
        archive_path: normalized,
        sha256: digest_hex(&bytes),
        size: bytes.len() as u64,
        data_b64: BASE64.encode(bytes),
    });
    *total_bytes = next_total;
    Ok(())
}

fn decrypt_payload(path: &Path, password: &str) -> Result<BackupPayload> {
    validate_password(password)?;
    let metadata = fs::metadata(path).map_err(|_| BackupError::Io("backup file could not be read"))?;
    if metadata.len() > MAX_ARCHIVE_BYTES + HEADER_LEN as u64 {
        return Err(BackupError::Format("backup file exceeds the safe archive limit"));
    }
    let bytes = fs::read(path).map_err(|_| BackupError::Io("backup file could not be read"))?;
    if bytes.len() < HEADER_LEN + TAG_LEN {
        return Err(BackupError::Format("backup file is truncated"));
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        return Err(BackupError::Format("backup file magic is invalid"));
    }
    let version_offset = MAGIC.len();
    let version = u16::from_le_bytes([bytes[version_offset], bytes[version_offset + 1]]);
    if version != FORMAT_VERSION {
        return Err(BackupError::Format("backup format version is unsupported"));
    }

    let salt_start = version_offset + 2;
    let nonce_start = salt_start + SALT_LEN;
    let length_start = nonce_start + NONCE_LEN;
    let salt: [u8; SALT_LEN] = bytes[salt_start..nonce_start]
        .try_into()
        .map_err(|_| BackupError::Format("backup salt is malformed"))?;
    let nonce: [u8; NONCE_LEN] = bytes[nonce_start..length_start]
        .try_into()
        .map_err(|_| BackupError::Format("backup nonce is malformed"))?;
    let cipher_len = u64::from_le_bytes(
        bytes[length_start..HEADER_LEN]
            .try_into()
            .map_err(|_| BackupError::Format("backup length is malformed"))?,
    );
    if cipher_len as usize != bytes.len() - HEADER_LEN {
        return Err(BackupError::Format("backup ciphertext length is invalid"));
    }

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(key.as_ref())
        .map_err(|_| BackupError::Crypto("backup cipher could not be initialized"))?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &bytes[HEADER_LEN..],
                aad: &bytes[..HEADER_LEN],
            },
        )
        .map_err(|_| BackupError::Crypto("backup password is incorrect or the backup was modified"))?;
    let payload: BackupPayload = serde_json::from_slice(&plaintext)
        .map_err(|_| BackupError::Format("decrypted backup payload is malformed"))?;
    if payload.format_version != FORMAT_VERSION {
        return Err(BackupError::Format("decrypted backup format version is unsupported"));
    }
    Ok(payload)
}

fn validate_payload(payload: &BackupPayload) -> Result<()> {
    if payload.entries.len() > MAX_ENTRIES {
        return Err(BackupError::Integrity("backup contains too many entries"));
    }
    let mut seen = HashSet::new();
    let mut total = 0_u64;
    for entry in &payload.entries {
        if !seen.insert(entry.archive_path.clone()) {
            return Err(BackupError::Integrity("backup contains duplicate archive paths"));
        }
        validated_relative_path(&entry.archive_path)?;
        let bytes = BASE64
            .decode(&entry.data_b64)
            .map_err(|_| BackupError::Format("backup file data is malformed"))?;
        if bytes.len() as u64 != entry.size || digest_hex(&bytes) != entry.sha256 {
            return Err(BackupError::Integrity("backup entry integrity verification failed"));
        }
        if entry.size > MAX_FILE_BYTES {
            return Err(BackupError::Integrity("backup entry exceeds the safe file-size limit"));
        }
        total = total
            .checked_add(entry.size)
            .ok_or(BackupError::Integrity("backup byte count overflowed"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(BackupError::Integrity("backup exceeds the safe total file-size limit"));
        }
    }
    Ok(())
}

fn summary_from_payload(payload: &BackupPayload, verified: bool) -> Result<BackupSummary> {
    let total_bytes = payload.entries.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(entry.size)
            .ok_or(BackupError::Integrity("backup byte count overflowed"))
    })?;
    Ok(BackupSummary {
        format_version: payload.format_version,
        created_at_ms: payload.created_at_ms,
        selections: payload.selections.clone(),
        entries: payload.entries.len(),
        total_bytes,
        verified,
    })
}

fn derive_key(password: &str, salt: &[u8; SALT_LEN]) -> Result<Zeroizing<[u8; 32]>> {
    let params = Params::new(ARGON2_MEMORY_KIB, ARGON2_ITERATIONS, ARGON2_LANES, Some(32))
        .map_err(|_| BackupError::Crypto("backup key-derivation parameters are invalid"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0_u8; 32]);
    argon2
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|_| BackupError::Crypto("backup key derivation failed"))?;
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
        return Err(BackupError::Integrity("backup contains an invalid relative path"));
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(BackupError::Integrity("backup contains an absolute path"));
    }
    for component in path.components() {
        if !matches!(component, PathComponent::Normal(_)) {
            return Err(BackupError::Integrity("backup contains a traversal-like path"));
        }
    }
    Ok(path.to_path_buf())
}

fn sanitize_segment(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control() && !matches!(character, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        .take(120)
        .collect::<String>()
        .trim()
        .to_owned()
}

fn validate_password(password: &str) -> Result<()> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(BackupError::InvalidInput("backup password must contain at least 12 characters"));
    }
    Ok(())
}

fn digest_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn nonempty_parent(path: &Path) -> Option<&Path> {
    path.parent().filter(|parent| !parent.as_os_str().is_empty())
}

fn temporary_sibling(destination: &Path) -> Result<PathBuf> {
    let parent = nonempty_parent(destination).unwrap_or_else(|| Path::new("."));
    let mut random = [0_u8; 8];
    OsRng.fill_bytes(&mut random);
    let token = random.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("dragonforge-backup");
    Ok(parent.join(format!(".{name}.{token}.tmp")))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{create_backup, inspect_backup, restore_backup, verify_backup};

    const PASSWORD: &str = "correct horse battery staple";

    #[test]
    fn encrypted_backup_round_trip_restores_files() {
        let dir = tempdir().expect("tempdir");
        let source = dir.path().join("source");
        fs::create_dir(&source).expect("source");
        fs::write(source.join("alpha.txt"), b"alpha").expect("alpha");
        fs::create_dir(source.join("nested")).expect("nested");
        fs::write(source.join("nested").join("beta.bin"), [1_u8, 2, 3, 4]).expect("beta");

        let backup = dir.path().join("sample.dfbackup");
        let created = create_backup(std::slice::from_ref(&source), &backup, PASSWORD).expect("create");
        assert_eq!(created.entries, 2);
        assert!(created.verified);

        let inspected = inspect_backup(&backup, PASSWORD).expect("inspect");
        assert_eq!(inspected.entries, 2);
        assert!(!inspected.verified);

        let verified = verify_backup(&backup, PASSWORD).expect("verify");
        assert!(verified.verified);

        let restored = dir.path().join("restored");
        let summary = restore_backup(&backup, PASSWORD, &restored).expect("restore");
        assert_eq!(summary.entries, 2);
        let root = fs::read_dir(&restored)
            .expect("restored root")
            .next()
            .expect("source root")
            .expect("source entry")
            .path();
        assert_eq!(fs::read(root.join("alpha.txt")).expect("read alpha"), b"alpha");
    }

    #[test]
    fn wrong_password_is_rejected() {
        let dir = tempdir().expect("tempdir");
        let source = dir.path().join("file.txt");
        fs::write(&source, b"secret").expect("source");
        let backup = dir.path().join("sample.dfbackup");
        create_backup(&[source], &backup, PASSWORD).expect("create");
        assert!(verify_backup(&backup, "this password is wrong").is_err());
    }

    #[test]
    fn modified_ciphertext_is_rejected() {
        let dir = tempdir().expect("tempdir");
        let source = dir.path().join("file.txt");
        fs::write(&source, b"secret").expect("source");
        let backup = dir.path().join("sample.dfbackup");
        create_backup(&[source], &backup, PASSWORD).expect("create");
        let mut bytes = fs::read(&backup).expect("backup");
        let last = bytes.len() - 1;
        bytes[last] ^= 0x55;
        fs::write(&backup, bytes).expect("tamper");
        assert!(verify_backup(&backup, PASSWORD).is_err());
    }

    #[test]
    fn restore_refuses_existing_destination() {
        let dir = tempdir().expect("tempdir");
        let source = dir.path().join("file.txt");
        fs::write(&source, b"secret").expect("source");
        let backup = dir.path().join("sample.dfbackup");
        create_backup(&[source], &backup, PASSWORD).expect("create");
        let restored = dir.path().join("restored");
        fs::create_dir(&restored).expect("existing");
        assert!(restore_backup(&backup, PASSWORD, &restored).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_sources_are_rejected() {
        use std::os::unix::fs::symlink;

        let dir = tempdir().expect("tempdir");
        let real = dir.path().join("real.txt");
        fs::write(&real, b"secret").expect("real");
        let link = dir.path().join("link.txt");
        symlink(&real, &link).expect("link");
        let backup = dir.path().join("sample.dfbackup");
        assert!(create_backup(&[link], &backup, PASSWORD).is_err());
    }
}
