use std::collections::HashSet;
use std::fs;
use std::path::{Component as PathComponent, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use dragonforge_core::{Platform, SuitePaths};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::error::{BackupError, Result};
use crate::format::{
    MAX_ARCHIVE_BYTES, MAX_ENTRIES, MAX_FILE_BYTES, MAX_PATH_CHARS, MAX_TOTAL_BYTES, NONCE_LEN,
    RECOVERY_EXTENSION, RECOVERY_FORMAT_VERSION, RECOVERY_HEADER_LEN, RECOVERY_MAGIC,
    RECOVERY_SCHEMA_VERSION, SALT_LEN, TAG_LEN,
};

const ARGON2_MEMORY_KIB: u32 = 65_536;
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_LANES: u32 = 1;
const MIN_PASSWORD_CHARS: usize = 12;
const MAX_REPAIR_JSON_BYTES: u64 = 4 * 1024 * 1024;
const MAX_REPAIR_DEPTH: usize = 32;
const MAX_REPAIR_BACKUPS: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryScope {
    Configuration,
    FullSuite,
}

impl RecoveryScope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Configuration => "configuration",
            Self::FullSuite => "full_suite",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecoverySummary {
    pub format_version: u16,
    pub schema_version: u16,
    pub original_schema_version: u16,
    pub migrated: bool,
    pub created_at_ms: u64,
    pub scope: RecoveryScope,
    pub source_platform: String,
    pub suite_version: String,
    pub entries: usize,
    pub total_bytes: u64,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecoveryRestoreSummary {
    pub config_root: String,
    pub data_root: Option<String>,
    pub entries: usize,
    pub total_bytes: u64,
    pub migrated_from_schema: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepairSummary {
    pub inspected_backups: usize,
    pub recovered_missing: usize,
    pub recovered_corrupt: usize,
    pub skipped_invalid_backups: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecoveryEntry {
    namespace: String,
    relative_path: String,
    sha256: String,
    size: u64,
    data_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecoveryPayloadV2 {
    schema_version: u16,
    created_at_ms: u64,
    scope: RecoveryScope,
    source_platform: String,
    suite_version: String,
    entries: Vec<RecoveryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecoveryPayloadV1 {
    schema_version: u16,
    created_at_ms: u64,
    scope: RecoveryScope,
    entries: Vec<RecoveryEntry>,
}

pub fn create_suite_recovery(
    destination: &Path,
    password: &str,
    scope: RecoveryScope,
) -> Result<RecoverySummary> {
    let paths = SuitePaths::discover()
        .map_err(|_| BackupError::Io("DragonForge suite paths could not be discovered"))?;
    create_suite_recovery_from_paths(
        &paths,
        destination,
        password,
        scope,
        env!("CARGO_PKG_VERSION"),
    )
}

pub fn create_suite_recovery_from_paths(
    paths: &SuitePaths,
    destination: &Path,
    password: &str,
    scope: RecoveryScope,
    suite_version: &str,
) -> Result<RecoverySummary> {
    validate_password(password)?;
    validate_destination(destination)?;
    let mut entries = Vec::new();
    let mut total_bytes = 0_u64;

    collect_namespace(
        paths.config_root(),
        "config",
        &mut entries,
        &mut total_bytes,
    )?;
    if scope == RecoveryScope::FullSuite {
        collect_namespace(paths.data_root(), "data", &mut entries, &mut total_bytes)?;
    }
    if entries.is_empty() {
        return Err(BackupError::InvalidInput(
            "no recoverable DragonForge state was found",
        ));
    }

    let payload = RecoveryPayloadV2 {
        schema_version: RECOVERY_SCHEMA_VERSION,
        created_at_ms: now_ms(),
        scope,
        source_platform: Platform::current().as_str().to_owned(),
        suite_version: bounded_label(suite_version, 64)?,
        entries,
    };
    validate_payload_v2(&payload)?;
    let plaintext = serde_json::to_vec(&payload)
        .map_err(|_| BackupError::Format("recovery payload could not be serialized"))?;
    write_encrypted_recovery(destination, password, &plaintext)?;
    verify_suite_recovery(destination, password)
}

pub fn inspect_suite_recovery(path: &Path, password: &str) -> Result<RecoverySummary> {
    let decoded = decrypt_recovery(path, password)?;
    let migrated = decode_and_migrate(&decoded)?;
    summary_from_payload(&migrated.payload, migrated.original_schema, false)
}

pub fn verify_suite_recovery(path: &Path, password: &str) -> Result<RecoverySummary> {
    let decoded = decrypt_recovery(path, password)?;
    let migrated = decode_and_migrate(&decoded)?;
    validate_payload_v2(&migrated.payload)?;
    summary_from_payload(&migrated.payload, migrated.original_schema, true)
}

pub fn restore_suite_recovery(
    path: &Path,
    password: &str,
    config_root: &Path,
    data_root: Option<&Path>,
) -> Result<RecoveryRestoreSummary> {
    let decoded = decrypt_recovery(path, password)?;
    let migrated = decode_and_migrate(&decoded)?;
    validate_payload_v2(&migrated.payload)?;
    validate_restore_roots(&migrated.payload, config_root, data_root)?;
    let config_root_existed = config_root.exists();
    let data_root_existed = data_root.is_some_and(Path::exists);

    let config_stage = temporary_sibling(config_root, "dfrestore")?;
    let data_stage = if migrated.payload.scope == RecoveryScope::FullSuite {
        Some(temporary_sibling(
            data_root.ok_or(BackupError::InvalidInput(
                "full-suite recovery requires a data root",
            ))?,
            "dfrestore",
        )?)
    } else {
        None
    };

    if config_stage.exists() {
        fs::remove_dir_all(&config_stage).map_err(|_| {
            BackupError::Io("stale configuration restore stage could not be removed")
        })?;
    }
    fs::create_dir_all(&config_stage)
        .map_err(|_| BackupError::Io("configuration restore stage could not be created"))?;

    if let Some(stage) = &data_stage {
        if stage.exists() {
            fs::remove_dir_all(stage)
                .map_err(|_| BackupError::Io("stale data restore stage could not be removed"))?;
        }
        fs::create_dir_all(stage)
            .map_err(|_| BackupError::Io("data restore stage could not be created"))?;
    }

    let result = (|| {
        for entry in &migrated.payload.entries {
            let relative = validated_relative_path(&entry.relative_path)?;
            let base = match entry.namespace.as_str() {
                "config" => &config_stage,
                "data" => data_stage.as_ref().ok_or(BackupError::Format(
                    "recovery package data namespace is inconsistent with its scope",
                ))?,
                _ => {
                    return Err(BackupError::Format(
                        "recovery package contains an unsupported namespace",
                    ));
                }
            };
            let target = base.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).map_err(|_| {
                    BackupError::Io("recovery restore directory could not be created")
                })?;
            }
            let bytes = BASE64
                .decode(&entry.data_b64)
                .map_err(|_| BackupError::Format("recovery file data is malformed"))?;
            if bytes.len() as u64 != entry.size || digest_hex(&bytes) != entry.sha256 {
                return Err(BackupError::Integrity(
                    "recovery entry integrity verification failed",
                ));
            }
            fs::write(&target, bytes)
                .map_err(|_| BackupError::Io("recovery entry could not be restored"))?;
        }

        if let Some(parent) = config_root.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| BackupError::Io("configuration restore parent could not be created"))?;
        }
        prepare_clean_restore_root(config_root)?;
        if fs::rename(&config_stage, config_root).is_err() {
            if config_root_existed && !config_root.exists() {
                let _ = fs::create_dir_all(config_root);
            }
            return Err(BackupError::Io(
                "configuration restore could not be finalized",
            ));
        }

        if let (Some(stage), Some(root)) = (&data_stage, data_root) {
            let data_result = (|| {
                if let Some(parent) = root.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|_| BackupError::Io("data restore parent could not be created"))?;
                }
                prepare_clean_restore_root(root)?;
                fs::rename(stage, root)
                    .map_err(|_| BackupError::Io("data restore could not be finalized"))
            })();
            if let Err(error) = data_result {
                let _ = fs::remove_dir_all(config_root);
                if config_root_existed {
                    let _ = fs::create_dir_all(config_root);
                }
                if data_root_existed && !root.exists() {
                    let _ = fs::create_dir_all(root);
                }
                return Err(error);
            }
        }
        Ok(())
    })();

    if let Err(error) = result {
        if config_stage.exists() {
            let _ = fs::remove_dir_all(&config_stage);
        }
        if let Some(stage) = &data_stage {
            if stage.exists() {
                let _ = fs::remove_dir_all(stage);
            }
        }
        return Err(error);
    }

    let total_bytes = migrated
        .payload
        .entries
        .iter()
        .try_fold(0_u64, |total, entry| total.checked_add(entry.size))
        .ok_or(BackupError::Integrity("recovery byte count overflowed"))?;

    Ok(RecoveryRestoreSummary {
        config_root: config_root.to_string_lossy().into_owned(),
        data_root: data_root.map(|path| path.to_string_lossy().into_owned()),
        entries: migrated.payload.entries.len(),
        total_bytes,
        migrated_from_schema: (migrated.original_schema != RECOVERY_SCHEMA_VERSION)
            .then_some(migrated.original_schema),
    })
}

pub fn repair_recoverable_json_state(paths: &SuitePaths) -> Result<RepairSummary> {
    let mut summary = RepairSummary {
        inspected_backups: 0,
        recovered_missing: 0,
        recovered_corrupt: 0,
        skipped_invalid_backups: 0,
    };
    for root in [paths.config_root(), paths.data_root()] {
        if root.exists() {
            repair_directory(root, &mut summary, 0)?;
        }
    }
    Ok(summary)
}

fn repair_directory(directory: &Path, summary: &mut RepairSummary, depth: usize) -> Result<()> {
    if depth > MAX_REPAIR_DEPTH {
        return Err(BackupError::InvalidInput(
            "suite state repair exceeded the safe directory-depth limit",
        ));
    }
    let mut children = fs::read_dir(directory)
        .map_err(|_| BackupError::Io("suite state directory could not be inspected"))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| BackupError::Io("suite state directory entries could not be inspected"))?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| BackupError::Io("suite state metadata could not be inspected"))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            repair_directory(&path, summary, depth.saturating_add(1))?;
            continue;
        }
        let name = path.file_name().and_then(|value| value.to_str()).unwrap_or("");
        if !name.ends_with(".json.bak") {
            continue;
        }
        summary.inspected_backups = summary.inspected_backups.saturating_add(1);
        if summary.inspected_backups > MAX_REPAIR_BACKUPS {
            return Err(BackupError::InvalidInput(
                "suite state repair exceeded the safe backup-file limit",
            ));
        }
        if metadata.len() > MAX_REPAIR_JSON_BYTES || !valid_json_file(&path) {
            summary.skipped_invalid_backups = summary.skipped_invalid_backups.saturating_add(1);
            continue;
        }

        let primary_name = name.trim_end_matches(".bak");
        let primary = path.with_file_name(primary_name);
        if !primary.exists() {
            fs::rename(&path, &primary)
                .map_err(|_| BackupError::Io("missing suite state could not be recovered"))?;
            summary.recovered_missing = summary.recovered_missing.saturating_add(1);
            continue;
        }

        let primary_metadata = fs::metadata(&primary)
            .map_err(|_| BackupError::Io("suite state metadata could not be read"))?;
        if primary_metadata.len() <= MAX_REPAIR_JSON_BYTES && valid_json_file(&primary) {
            continue;
        }

        let invalid = append_suffix(&primary, ".invalid");
        if invalid.exists() {
            fs::remove_file(&invalid)
                .map_err(|_| BackupError::Io("stale suite state quarantine could not be removed"))?;
        }
        fs::rename(&primary, &invalid)
            .map_err(|_| BackupError::Io("corrupt suite state could not be quarantined"))?;
        fs::rename(&path, &primary)
            .map_err(|_| BackupError::Io("suite state backup could not be recovered"))?;
        summary.recovered_corrupt = summary.recovered_corrupt.saturating_add(1);
    }
    Ok(())
}

fn valid_json_file(path: &Path) -> bool {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .is_some()
}

fn collect_namespace(
    root: &Path,
    namespace: &str,
    entries: &mut Vec<RecoveryEntry>,
    total_bytes: &mut u64,
) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(root)
        .map_err(|_| BackupError::Io("suite recovery root could not be read"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(BackupError::InvalidInput(
            "suite recovery roots must be ordinary directories",
        ));
    }
    collect_recovery_directory(root, root, namespace, entries, total_bytes)
}

fn collect_recovery_directory(
    root: &Path,
    directory: &Path,
    namespace: &str,
    entries: &mut Vec<RecoveryEntry>,
    total_bytes: &mut u64,
) -> Result<()> {
    let mut children = fs::read_dir(directory)
        .map_err(|_| BackupError::Io("suite recovery directory could not be read"))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| BackupError::Io("suite recovery directory entries could not be read"))?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| BackupError::Io("suite recovery entry metadata could not be read"))?;
        if metadata.file_type().is_symlink() {
            return Err(BackupError::InvalidInput(
                "symbolic links inside suite recovery roots are not allowed",
            ));
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| BackupError::InvalidInput("suite recovery path escaped its root"))?;
        let normalized = normalize_relative_path(relative)?;
        if should_exclude_recovery_entry(&normalized) {
            continue;
        }
        if metadata.is_dir() {
            collect_recovery_directory(root, &path, namespace, entries, total_bytes)?;
        } else if metadata.is_file() {
            collect_recovery_file(
                &path,
                namespace,
                &normalized,
                entries,
                total_bytes,
            )?;
        }
    }
    Ok(())
}

fn collect_recovery_file(
    source: &Path,
    namespace: &str,
    relative_path: &str,
    entries: &mut Vec<RecoveryEntry>,
    total_bytes: &mut u64,
) -> Result<()> {
    if entries.len() >= MAX_ENTRIES {
        return Err(BackupError::InvalidInput(
            "recovery package contains too many files",
        ));
    }
    let metadata = fs::metadata(source)
        .map_err(|_| BackupError::Io("recovery source file metadata could not be read"))?;
    if metadata.len() > MAX_FILE_BYTES {
        return Err(BackupError::InvalidInput(
            "a recovery source file exceeds the safe size limit",
        ));
    }
    let next_total = total_bytes
        .checked_add(metadata.len())
        .ok_or(BackupError::InvalidInput("recovery package size overflowed"))?;
    if next_total > MAX_TOTAL_BYTES {
        return Err(BackupError::InvalidInput(
            "recovery package exceeds the safe total file-size limit",
        ));
    }

    let bytes =
        fs::read(source).map_err(|_| BackupError::Io("recovery source file could not be read"))?;
    entries.push(RecoveryEntry {
        namespace: namespace.to_owned(),
        relative_path: relative_path.to_owned(),
        sha256: digest_hex(&bytes),
        size: bytes.len() as u64,
        data_b64: BASE64.encode(bytes),
    });
    *total_bytes = next_total;
    Ok(())
}

fn should_exclude_recovery_entry(relative_path: &str) -> bool {
    let lower = relative_path.to_ascii_lowercase();
    let filename = lower.rsplit('/').next().unwrap_or(&lower);
    filename == "agent-session.key"
        || filename == "agent-runtime.json"
        || filename == "agent.lock"
        || filename.ends_with(".tmp")
        || filename.ends_with(".lock")
        || filename.ends_with(".bak")
        || filename.ends_with(".invalid")
        || lower.starts_with("cache/")
        || lower.contains("/cache/")
        || lower.starts_with("logs/")
        || lower.contains("/logs/")
        || lower.starts_with("support-bundles/")
        || lower.contains("/support-bundles/")
}

fn validate_payload_v2(payload: &RecoveryPayloadV2) -> Result<()> {
    if payload.schema_version != RECOVERY_SCHEMA_VERSION
        || payload.source_platform.is_empty()
        || payload.source_platform.len() > 32
        || payload.suite_version.is_empty()
        || payload.suite_version.len() > 64
        || payload.entries.len() > MAX_ENTRIES
    {
        return Err(BackupError::Format(
            "recovery package metadata is incompatible",
        ));
    }

    let mut seen = HashSet::new();
    let mut total = 0_u64;
    for entry in &payload.entries {
        if !matches!(entry.namespace.as_str(), "config" | "data")
            || (payload.scope == RecoveryScope::Configuration && entry.namespace == "data")
            || !seen.insert((entry.namespace.clone(), entry.relative_path.clone()))
        {
            return Err(BackupError::Integrity(
                "recovery package contains invalid or duplicate entries",
            ));
        }
        validated_relative_path(&entry.relative_path)?;
        if entry.size > MAX_FILE_BYTES {
            return Err(BackupError::Integrity(
                "recovery entry exceeds the safe file-size limit",
            ));
        }
        let bytes = BASE64
            .decode(&entry.data_b64)
            .map_err(|_| BackupError::Format("recovery file data is malformed"))?;
        if bytes.len() as u64 != entry.size || digest_hex(&bytes) != entry.sha256 {
            return Err(BackupError::Integrity(
                "recovery entry integrity verification failed",
            ));
        }
        total = total
            .checked_add(entry.size)
            .ok_or(BackupError::Integrity("recovery byte count overflowed"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(BackupError::Integrity(
                "recovery package exceeds the safe total file-size limit",
            ));
        }
    }
    Ok(())
}

struct MigratedPayload {
    original_schema: u16,
    payload: RecoveryPayloadV2,
}

fn decode_and_migrate(bytes: &[u8]) -> Result<MigratedPayload> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|_| BackupError::Format("decrypted recovery payload is malformed"))?;
    let schema = value
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or(BackupError::Format(
            "recovery package schema version is missing",
        ))?;

    match schema {
        1 => {
            let legacy: RecoveryPayloadV1 = serde_json::from_value(value)
                .map_err(|_| BackupError::Format("legacy recovery payload is malformed"))?;
            Ok(MigratedPayload {
                original_schema: 1,
                payload: RecoveryPayloadV2 {
                    schema_version: RECOVERY_SCHEMA_VERSION,
                    created_at_ms: legacy.created_at_ms,
                    scope: legacy.scope,
                    source_platform: "legacy-unknown".to_owned(),
                    suite_version: "legacy-unknown".to_owned(),
                    entries: legacy.entries,
                },
            })
        }
        RECOVERY_SCHEMA_VERSION => {
            let payload: RecoveryPayloadV2 = serde_json::from_value(value)
                .map_err(|_| BackupError::Format("recovery payload is malformed"))?;
            Ok(MigratedPayload {
                original_schema: RECOVERY_SCHEMA_VERSION,
                payload,
            })
        }
        _ => Err(BackupError::Format(
            "recovery package schema version is unsupported",
        )),
    }
}

fn summary_from_payload(
    payload: &RecoveryPayloadV2,
    original_schema: u16,
    verified: bool,
) -> Result<RecoverySummary> {
    let total_bytes = payload.entries.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(entry.size)
            .ok_or(BackupError::Integrity("recovery byte count overflowed"))
    })?;
    Ok(RecoverySummary {
        format_version: RECOVERY_FORMAT_VERSION,
        schema_version: RECOVERY_SCHEMA_VERSION,
        original_schema_version: original_schema,
        migrated: original_schema != RECOVERY_SCHEMA_VERSION,
        created_at_ms: payload.created_at_ms,
        scope: payload.scope,
        source_platform: payload.source_platform.clone(),
        suite_version: payload.suite_version.clone(),
        entries: payload.entries.len(),
        total_bytes,
        verified,
    })
}

fn validate_destination(destination: &Path) -> Result<()> {
    if destination.exists() {
        return Err(BackupError::Conflict(
            "recovery package destination already exists",
        ));
    }
    let valid_extension = destination
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(RECOVERY_EXTENSION));
    if !valid_extension {
        return Err(BackupError::InvalidInput(
            "recovery package must use the .dfrecovery extension",
        ));
    }
    Ok(())
}

fn validate_restore_roots(
    payload: &RecoveryPayloadV2,
    config_root: &Path,
    data_root: Option<&Path>,
) -> Result<()> {
    if !config_root.is_absolute() {
        return Err(BackupError::InvalidInput(
            "configuration recovery root must be an absolute path",
        ));
    }
    ensure_clean_restore_root(config_root)?;
    if payload.scope == RecoveryScope::FullSuite {
        let data_root = data_root.ok_or(BackupError::InvalidInput(
            "full-suite recovery requires a data root",
        ))?;
        if !data_root.is_absolute() {
            return Err(BackupError::InvalidInput(
                "data recovery root must be an absolute path",
            ));
        }
        if data_root == config_root
            || data_root.starts_with(config_root)
            || config_root.starts_with(data_root)
        {
            return Err(BackupError::InvalidInput(
                "configuration and data recovery roots must not overlap",
            ));
        }
        ensure_clean_restore_root(data_root)?;
    }
    Ok(())
}

fn ensure_clean_restore_root(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| BackupError::Io("recovery destination metadata could not be read"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(BackupError::Conflict(
            "recovery destination must be a missing or empty ordinary directory",
        ));
    }
    let mut entries = fs::read_dir(path)
        .map_err(|_| BackupError::Io("recovery destination could not be inspected"))?;
    if entries.next().is_some() {
        return Err(BackupError::Conflict(
            "recovery destination must be empty to prevent overwriting existing state",
        ));
    }
    Ok(())
}

fn prepare_clean_restore_root(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir(path)
            .map_err(|_| BackupError::Io("empty recovery destination could not be staged"))?;
    }
    Ok(())
}

fn write_encrypted_recovery(destination: &Path, password: &str, plaintext: &[u8]) -> Result<()> {
    if plaintext.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err(BackupError::InvalidInput(
            "recovery payload exceeds the safe archive limit",
        ));
    }

    let mut salt = [0_u8; SALT_LEN];
    let mut nonce_bytes = [0_u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key[..])
        .map_err(|_| BackupError::Crypto("recovery cipher could not be initialized"))?;
    let cipher_len = plaintext
        .len()
        .checked_add(TAG_LEN)
        .ok_or(BackupError::InvalidInput("recovery payload is too large"))?
        as u64;
    let aad = build_header(&salt, &nonce_bytes, cipher_len);
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| BackupError::Crypto("recovery encryption failed"))?;

    if let Some(parent) = destination.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|_| BackupError::Io("recovery destination directory could not be created"))?;
    }
    let temporary = temporary_sibling(destination, "tmp")?;
    fs::write(&temporary, [&aad[..], &ciphertext].concat())
        .map_err(|_| BackupError::Io("encrypted recovery package could not be written"))?;
    if fs::rename(&temporary, destination).is_err() {
        let _ = fs::remove_file(&temporary);
        return Err(BackupError::Io(
            "encrypted recovery package could not be finalized",
        ));
    }
    Ok(())
}

fn decrypt_recovery(path: &Path, password: &str) -> Result<Vec<u8>> {
    validate_password(password)?;
    let metadata =
        fs::metadata(path).map_err(|_| BackupError::Io("recovery package could not be read"))?;
    if metadata.len() > MAX_ARCHIVE_BYTES + RECOVERY_HEADER_LEN as u64 {
        return Err(BackupError::Format(
            "recovery package exceeds the safe archive limit",
        ));
    }
    let bytes =
        fs::read(path).map_err(|_| BackupError::Io("recovery package could not be read"))?;
    if bytes.len() < RECOVERY_HEADER_LEN + TAG_LEN {
        return Err(BackupError::Format("recovery package is truncated"));
    }
    if &bytes[..RECOVERY_MAGIC.len()] != RECOVERY_MAGIC {
        return Err(BackupError::Format("recovery package magic is invalid"));
    }

    let version_offset = RECOVERY_MAGIC.len();
    let version = u16::from_le_bytes([bytes[version_offset], bytes[version_offset + 1]]);
    if version != RECOVERY_FORMAT_VERSION {
        return Err(BackupError::Format(
            "recovery package format version is unsupported",
        ));
    }

    let salt_start = version_offset + 2;
    let nonce_start = salt_start + SALT_LEN;
    let length_start = nonce_start + NONCE_LEN;
    let salt: [u8; SALT_LEN] = bytes[salt_start..nonce_start]
        .try_into()
        .map_err(|_| BackupError::Format("recovery salt is malformed"))?;
    let nonce: [u8; NONCE_LEN] = bytes[nonce_start..length_start]
        .try_into()
        .map_err(|_| BackupError::Format("recovery nonce is malformed"))?;
    let cipher_len = u64::from_le_bytes(
        bytes[length_start..RECOVERY_HEADER_LEN]
            .try_into()
            .map_err(|_| BackupError::Format("recovery length is malformed"))?,
    );
    if cipher_len as usize != bytes.len() - RECOVERY_HEADER_LEN {
        return Err(BackupError::Format(
            "recovery ciphertext length is invalid",
        ));
    }

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(key.as_ref())
        .map_err(|_| BackupError::Crypto("recovery cipher could not be initialized"))?;
    cipher
        .decrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &bytes[RECOVERY_HEADER_LEN..],
                aad: &bytes[..RECOVERY_HEADER_LEN],
            },
        )
        .map_err(|_| {
            BackupError::Crypto(
                "recovery password is incorrect or the recovery package was modified",
            )
        })
}

fn derive_key(password: &str, salt: &[u8; SALT_LEN]) -> Result<Zeroizing<[u8; 32]>> {
    validate_password(password)?;
    let params = Params::new(ARGON2_MEMORY_KIB, ARGON2_ITERATIONS, ARGON2_LANES, Some(32))
        .map_err(|_| BackupError::Crypto("recovery key-derivation parameters are invalid"))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0_u8; 32]);
    argon2
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|_| BackupError::Crypto("recovery key derivation failed"))?;
    Ok(key)
}

fn build_header(salt: &[u8; SALT_LEN], nonce: &[u8; NONCE_LEN], cipher_len: u64) -> Vec<u8> {
    let mut header = Vec::with_capacity(RECOVERY_HEADER_LEN);
    header.extend_from_slice(RECOVERY_MAGIC);
    header.extend_from_slice(&RECOVERY_FORMAT_VERSION.to_le_bytes());
    header.extend_from_slice(salt);
    header.extend_from_slice(nonce);
    header.extend_from_slice(&cipher_len.to_le_bytes());
    header
}

fn validate_password(password: &str) -> Result<()> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(BackupError::InvalidInput(
            "recovery password must contain at least 12 characters",
        ));
    }
    Ok(())
}

fn normalize_relative_path(path: &Path) -> Result<String> {
    let text = path
        .to_str()
        .ok_or(BackupError::InvalidInput(
            "recovery path could not be represented safely",
        ))?
        .replace('\\', "/");
    validated_relative_path(&text)?;
    Ok(text)
}

fn validated_relative_path(value: &str) -> Result<PathBuf> {
    if value.is_empty() || value.chars().count() > MAX_PATH_CHARS {
        return Err(BackupError::Integrity(
            "recovery package contains an invalid relative path",
        ));
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(BackupError::Integrity(
            "recovery package contains an absolute path",
        ));
    }
    for component in path.components() {
        if !matches!(component, PathComponent::Normal(_)) {
            return Err(BackupError::Integrity(
                "recovery package contains a traversal-like path",
            ));
        }
    }
    Ok(path.to_path_buf())
}

fn bounded_label(value: &str, max_chars: usize) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > max_chars
        || trimmed
            .chars()
            .any(|character| matches!(character, '\r' | '\n' | '\0'))
    {
        return Err(BackupError::InvalidInput(
            "recovery metadata label is invalid",
        ));
    }
    Ok(trimmed.to_owned())
}

fn digest_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

fn temporary_sibling(path: &Path, tag: &str) -> Result<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut random = [0_u8; 8];
    OsRng.fill_bytes(&mut random);
    let token = random
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("dragonforge-recovery");
    Ok(parent.join(format!(".{name}.{tag}.{token}")))
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
    use super::*;
    use tempfile::tempdir;

    const PASSWORD: &str = "correct horse battery staple";

    fn test_paths(root: &Path) -> SuitePaths {
        SuitePaths::from_roots(
            root.join("config"),
            root.join("data"),
            root.join("cache"),
        )
    }

    #[test]
    fn configuration_recovery_round_trip_restores_clean_root() {
        let dir = tempdir().expect("tempdir");
        let paths = test_paths(dir.path());
        fs::create_dir_all(paths.config_root().join("security-center")).expect("config");
        fs::write(
            paths.config_root().join("security-center/settings.json"),
            br#"{"version":1}"#,
        )
        .expect("write");

        let package = dir.path().join("suite.dfrecovery");
        let created = create_suite_recovery_from_paths(
            &paths,
            &package,
            PASSWORD,
            RecoveryScope::Configuration,
            "0.1.0",
        )
        .expect("create");
        assert_eq!(created.entries, 1);
        assert!(created.verified);

        let restore = dir.path().join("clean-config");
        let restored =
            restore_suite_recovery(&package, PASSWORD, &restore, None).expect("restore");
        assert_eq!(restored.entries, 1);
        assert_eq!(
            fs::read(restore.join("security-center/settings.json")).expect("read"),
            br#"{"version":1}"#
        );
    }

    #[test]
    fn full_suite_recovery_excludes_runtime_and_transient_state() {
        let dir = tempdir().expect("tempdir");
        let paths = test_paths(dir.path());
        fs::create_dir_all(paths.config_root().join("security-center")).expect("config");
        fs::create_dir_all(paths.data_root().join("agent")).expect("data");
        fs::write(
            paths.config_root().join("security-center/settings.json"),
            b"{}",
        )
        .expect("settings");
        fs::write(paths.data_root().join("agent/scheduled-automation-v1.json"), b"{}")
            .expect("automation");
        fs::write(paths.data_root().join("agent/agent-session.key"), b"secret")
            .expect("session");
        fs::write(paths.data_root().join("agent/agent-runtime.json"), b"runtime")
            .expect("runtime");
        fs::write(paths.data_root().join("agent/state.json.tmp"), b"temp")
            .expect("temp");

        let package = dir.path().join("suite.dfrecovery");
        let created = create_suite_recovery_from_paths(
            &paths,
            &package,
            PASSWORD,
            RecoveryScope::FullSuite,
            "0.1.0",
        )
        .expect("create");
        assert_eq!(created.entries, 2);

        let config = dir.path().join("restored-config");
        let data = dir.path().join("restored-data");
        restore_suite_recovery(&package, PASSWORD, &config, Some(&data)).expect("restore");
        assert!(config.join("security-center/settings.json").exists());
        assert!(data.join("agent/scheduled-automation-v1.json").exists());
        assert!(!data.join("agent/agent-session.key").exists());
        assert!(!data.join("agent/agent-runtime.json").exists());
        assert!(!data.join("agent/state.json.tmp").exists());
    }

    #[test]
    fn clean_machine_restore_refuses_existing_state() {
        let dir = tempdir().expect("tempdir");
        let paths = test_paths(dir.path());
        fs::create_dir_all(paths.config_root()).expect("config");
        fs::write(paths.config_root().join("settings.json"), b"{}").expect("write");
        let package = dir.path().join("suite.dfrecovery");
        create_suite_recovery_from_paths(
            &paths,
            &package,
            PASSWORD,
            RecoveryScope::Configuration,
            "0.1.0",
        )
        .expect("create");

        let target = dir.path().join("existing");
        fs::create_dir(&target).expect("target");
        fs::write(target.join("keep.txt"), b"keep").expect("keep");
        assert!(restore_suite_recovery(&package, PASSWORD, &target, None).is_err());
        assert_eq!(fs::read(target.join("keep.txt")).expect("keep"), b"keep");
    }

    #[test]
    fn recovery_package_requires_explicit_extension() {
        let dir = tempdir().expect("tempdir");
        let paths = test_paths(dir.path());
        fs::create_dir_all(paths.config_root()).expect("config");
        fs::write(paths.config_root().join("settings.json"), b"{}").expect("write");
        assert!(
            create_suite_recovery_from_paths(
                &paths,
                &dir.path().join("suite.backup"),
                PASSWORD,
                RecoveryScope::Configuration,
                "0.1.0",
            )
            .is_err()
        );
    }

    #[test]
    fn full_suite_restore_rejects_overlapping_roots() {
        let dir = tempdir().expect("tempdir");
        let paths = test_paths(dir.path());
        fs::create_dir_all(paths.config_root()).expect("config");
        fs::create_dir_all(paths.data_root()).expect("data");
        fs::write(paths.config_root().join("settings.json"), b"{}").expect("config file");
        fs::write(paths.data_root().join("state.json"), b"{}").expect("data file");

        let package = dir.path().join("suite.dfrecovery");
        create_suite_recovery_from_paths(
            &paths,
            &package,
            PASSWORD,
            RecoveryScope::FullSuite,
            "0.1.0",
        )
        .expect("create");

        let config = dir.path().join("restore");
        let nested_data = config.join("data");
        assert!(
            restore_suite_recovery(&package, PASSWORD, &config, Some(&nested_data)).is_err()
        );
        assert!(!config.exists());
    }

    #[test]
    fn corrupted_ciphertext_is_rejected_before_restore() {
        let dir = tempdir().expect("tempdir");
        let paths = test_paths(dir.path());
        fs::create_dir_all(paths.config_root()).expect("config");
        fs::write(paths.config_root().join("settings.json"), b"{}").expect("write");
        let package = dir.path().join("suite.dfrecovery");
        create_suite_recovery_from_paths(
            &paths,
            &package,
            PASSWORD,
            RecoveryScope::Configuration,
            "0.1.0",
        )
        .expect("create");

        let mut bytes = fs::read(&package).expect("read");
        let last = bytes.len() - 1;
        bytes[last] ^= 0x44;
        fs::write(&package, bytes).expect("tamper");
        assert!(verify_suite_recovery(&package, PASSWORD).is_err());
    }

    #[test]
    fn legacy_schema_is_migrated_in_memory() {
        let dir = tempdir().expect("tempdir");
        let package = dir.path().join("legacy.dfrecovery");
        let legacy = RecoveryPayloadV1 {
            schema_version: 1,
            created_at_ms: 7,
            scope: RecoveryScope::Configuration,
            entries: Vec::new(),
        };
        let encoded = serde_json::to_vec(&legacy).expect("encode");
        write_encrypted_recovery(&package, PASSWORD, &encoded).expect("write");
        let summary = verify_suite_recovery(&package, PASSWORD).expect("verify");
        assert!(summary.migrated);
        assert_eq!(summary.original_schema_version, 1);
        assert_eq!(summary.schema_version, RECOVERY_SCHEMA_VERSION);
        assert_eq!(summary.source_platform, "legacy-unknown");
    }

    #[test]
    fn future_schema_fails_closed() {
        let dir = tempdir().expect("tempdir");
        let package = dir.path().join("future.dfrecovery");
        let encoded =
            br#"{"schemaVersion":999,"createdAtMs":1,"scope":"configuration","entries":[]}"#;
        write_encrypted_recovery(&package, PASSWORD, encoded).expect("write");
        assert!(verify_suite_recovery(&package, PASSWORD).is_err());
    }

    #[test]
    fn json_backup_repair_recovers_missing_and_corrupt_state() {
        let dir = tempdir().expect("tempdir");
        let paths = test_paths(dir.path());
        let component = paths.config_root().join("security-center");
        fs::create_dir_all(&component).expect("component");

        let missing = component.join("missing.json");
        fs::write(append_suffix(&missing, ".bak"), br#"{"version":1}"#).expect("backup");

        let corrupt = component.join("corrupt.json");
        fs::write(&corrupt, b"{not-json").expect("corrupt");
        fs::write(append_suffix(&corrupt, ".bak"), br#"{"version":1}"#).expect("backup");

        let invalid_backup = component.join("invalid.json");
        fs::write(append_suffix(&invalid_backup, ".bak"), b"{broken").expect("bad backup");

        let summary = repair_recoverable_json_state(&paths).expect("repair");
        assert_eq!(summary.inspected_backups, 3);
        assert_eq!(summary.recovered_missing, 1);
        assert_eq!(summary.recovered_corrupt, 1);
        assert_eq!(summary.skipped_invalid_backups, 1);
        assert!(missing.exists());
        assert!(corrupt.exists());
        assert!(append_suffix(&corrupt, ".invalid").exists());
    }
}
