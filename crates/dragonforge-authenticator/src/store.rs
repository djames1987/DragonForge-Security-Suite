use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use percent_encoding::percent_decode_str;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

use crate::error::{AuthenticatorError, Result};
use crate::otp::{GeneratedCode, OtpAlgorithm, OtpKind, generate_hotp, generate_totp, normalize_secret};

const MAGIC: &[u8; 4] = b"DFA1";
const FORMAT_VERSION: u16 = 1;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const HEADER_LEN: usize = 4 + 2 + 4 + 4 + 4 + SALT_LEN + NONCE_LEN;
const ARGON_MEMORY_KIB: u32 = 65_536;
const ARGON_ITERATIONS: u32 = 3;
const ARGON_LANES: u32 = 1;
const MAX_PLAINTEXT_STORE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_STORE_FILE_BYTES: u64 = MAX_PLAINTEXT_STORE_BYTES + HEADER_LEN as u64 + 16;
const MAX_ACCOUNTS: usize = 500;
const MAX_RECOVERY_CODES: usize = 100;
const MAX_LABEL_LEN: usize = 160;
const MAX_ISSUER_LEN: usize = 120;
const MAX_RECOVERY_CODE_LEN: usize = 256;

pub struct NewAccount {
    pub label: String,
    pub issuer: String,
    pub secret_base32: String,
    pub algorithm: OtpAlgorithm,
    pub digits: u32,
    pub kind: OtpKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AccountView {
    pub id: String,
    pub label: String,
    pub issuer: String,
    pub algorithm: OtpAlgorithm,
    pub digits: u32,
    pub kind: OtpKind,
    pub recovery_code_count: usize,
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct CodeView {
    pub account_id: String,
    pub code: String,
    pub valid_for_seconds: Option<u64>,
    pub counter: u64,
}

#[derive(Serialize, Deserialize)]
struct StoredAccount {
    id: String,
    label: String,
    issuer: String,
    secret_base32: String,
    algorithm: OtpAlgorithm,
    digits: u32,
    kind: OtpKind,
    recovery_codes: Vec<String>,
}

impl Drop for StoredAccount {
    fn drop(&mut self) {
        self.secret_base32.zeroize();
        for code in &mut self.recovery_codes {
            code.zeroize();
        }
    }
}

impl StoredAccount {
    fn view(&self) -> AccountView {
        AccountView {
            id: self.id.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            algorithm: self.algorithm,
            digits: self.digits,
            kind: self.kind,
            recovery_code_count: self.recovery_codes.len(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct StoreData {
    version: u16,
    accounts: Vec<StoredAccount>,
}

pub fn create_store(path: impl AsRef<Path>, password: &[u8]) -> Result<()> {
    validate_password(password)?;
    let path = path.as_ref();
    recover_interrupted_replace(path)?;
    if path.exists() {
        return Err(AuthenticatorError::StoreExists);
    }
    let store = StoreData {
        version: FORMAT_VERSION,
        accounts: Vec::new(),
    };
    save_store(path, password, &store)
}

pub fn change_password(
    path: impl AsRef<Path>,
    current_password: &[u8],
    new_password: &[u8],
) -> Result<()> {
    validate_password(new_password)?;
    let path = path.as_ref();
    let store = load_store(path, current_password)?;
    save_store(path, new_password, &store)
}

pub fn list_accounts(path: impl AsRef<Path>, password: &[u8]) -> Result<Vec<AccountView>> {
    let store = load_store(path.as_ref(), password)?;
    Ok(store.accounts.iter().map(StoredAccount::view).collect())
}

pub fn add_account(
    path: impl AsRef<Path>,
    password: &[u8],
    account: NewAccount,
) -> Result<AccountView> {
    let path = path.as_ref();
    let mut normalized = validate_new_account(account)?;
    let mut store = match load_store(path, password) {
        Ok(store) => store,
        Err(error) => {
            normalized.secret_base32.zeroize();
            return Err(error);
        }
    };
    if store.accounts.len() >= MAX_ACCOUNTS {
        normalized.secret_base32.zeroize();
        return Err(AuthenticatorError::TooManyAccounts);
    }

    let secret_base32 = std::mem::take(&mut normalized.secret_base32);
    let stored = StoredAccount {
        id: Uuid::new_v4().to_string(),
        label: normalized.label,
        issuer: normalized.issuer,
        secret_base32,
        algorithm: normalized.algorithm,
        digits: normalized.digits,
        kind: normalized.kind,
        recovery_codes: Vec::new(),
    };
    let view = stored.view();
    store.accounts.push(stored);
    save_store(path, password, &store)?;
    Ok(view)
}

pub fn import_otpauth_uri(
    path: impl AsRef<Path>,
    password: &[u8],
    uri: &str,
) -> Result<AccountView> {
    add_account(path, password, parse_otpauth_uri(uri)?)
}

pub fn remove_account(path: impl AsRef<Path>, password: &[u8], id: &str) -> Result<()> {
    let path = path.as_ref();
    let mut store = load_store(path, password)?;
    let original_len = store.accounts.len();
    store.accounts.retain(|account| account.id != id);
    if store.accounts.len() == original_len {
        return Err(AuthenticatorError::AccountNotFound);
    }
    save_store(path, password, &store)
}

pub fn generate_code(
    path: impl AsRef<Path>,
    password: &[u8],
    id: &str,
    timestamp_seconds: u64,
) -> Result<CodeView> {
    let store = load_store(path.as_ref(), password)?;
    let account = store
        .accounts
        .iter()
        .find(|account| account.id == id)
        .ok_or(AuthenticatorError::AccountNotFound)?;

    let generated = match account.kind {
        OtpKind::Totp { period } => generate_totp(
            &account.secret_base32,
            account.algorithm,
            account.digits,
            period,
            timestamp_seconds,
        )?,
        OtpKind::Hotp { counter } => {
            generate_hotp(&account.secret_base32, account.algorithm, account.digits, counter)?
        }
    };
    Ok(code_view(id, generated))
}

pub fn consume_hotp(path: impl AsRef<Path>, password: &[u8], id: &str) -> Result<CodeView> {
    let path = path.as_ref();
    let mut store = load_store(path, password)?;
    let account = store
        .accounts
        .iter_mut()
        .find(|account| account.id == id)
        .ok_or(AuthenticatorError::AccountNotFound)?;

    let counter = match account.kind {
        OtpKind::Hotp { counter } => counter,
        OtpKind::Totp { .. } => return Err(AuthenticatorError::InvalidAccount),
    };
    let generated =
        generate_hotp(&account.secret_base32, account.algorithm, account.digits, counter)?;
    account.kind = OtpKind::Hotp {
        counter: counter
            .checked_add(1)
            .ok_or(AuthenticatorError::InvalidAccount)?,
    };
    let view = code_view(id, generated);
    save_store(path, password, &store)?;
    Ok(view)
}

pub fn set_recovery_codes(
    path: impl AsRef<Path>,
    password: &[u8],
    id: &str,
    codes: Vec<String>,
) -> Result<usize> {
    if codes.len() > MAX_RECOVERY_CODES {
        return Err(AuthenticatorError::TooManyRecoveryCodes);
    }
    let mut normalized = Vec::with_capacity(codes.len());
    for code in codes {
        let value = code.trim().to_owned();
        if value.is_empty()
            || value.len() > MAX_RECOVERY_CODE_LEN
            || value.contains('')
            || value.contains('
')
        {
            return Err(AuthenticatorError::InvalidRecoveryCode);
        }
        normalized.push(value);
    }

    let path = path.as_ref();
    let mut store = load_store(path, password)?;
    let account = store
        .accounts
        .iter_mut()
        .find(|account| account.id == id)
        .ok_or(AuthenticatorError::AccountNotFound)?;
    for code in &mut account.recovery_codes {
        code.zeroize();
    }
    account.recovery_codes = normalized;
    let count = account.recovery_codes.len();
    save_store(path, password, &store)?;
    Ok(count)
}

pub fn reveal_recovery_codes(
    path: impl AsRef<Path>,
    password: &[u8],
    id: &str,
) -> Result<Vec<String>> {
    let store = load_store(path.as_ref(), password)?;
    let account = store
        .accounts
        .iter()
        .find(|account| account.id == id)
        .ok_or(AuthenticatorError::AccountNotFound)?;
    Ok(account.recovery_codes.clone())
}

pub fn parse_otpauth_uri(uri: &str) -> Result<NewAccount> {
    let parsed = Url::parse(uri).map_err(|_| AuthenticatorError::InvalidOtpUri)?;
    if parsed.scheme() != "otpauth" {
        return Err(AuthenticatorError::InvalidOtpUri);
    }

    let host = parsed.host_str().ok_or(AuthenticatorError::InvalidOtpUri)?;
    let label = percent_decode_str(parsed.path().trim_start_matches('/'))
        .decode_utf8()
        .map_err(|_| AuthenticatorError::InvalidOtpUri)?
        .trim()
        .to_owned();
    if label.is_empty() {
        return Err(AuthenticatorError::InvalidOtpUri);
    }

    let mut secret: Option<Zeroizing<String>> = None;
    let mut issuer = None;
    let mut algorithm = OtpAlgorithm::Sha1;
    let mut digits = 6_u32;
    let mut period = 30_u64;
    let mut counter = None;

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "secret" => secret = Some(Zeroizing::new(value.into_owned())),
            "issuer" => issuer = Some(value.into_owned()),
            "algorithm" => {
                algorithm = match value.to_ascii_uppercase().as_str() {
                    "SHA1" => OtpAlgorithm::Sha1,
                    "SHA256" => OtpAlgorithm::Sha256,
                    "SHA512" => OtpAlgorithm::Sha512,
                    _ => return Err(AuthenticatorError::InvalidOtpUri),
                };
            }
            "digits" => {
                digits = value
                    .parse()
                    .map_err(|_| AuthenticatorError::InvalidOtpUri)?;
            }
            "period" => {
                period = value
                    .parse()
                    .map_err(|_| AuthenticatorError::InvalidOtpUri)?;
            }
            "counter" => {
                counter = Some(
                    value
                        .parse()
                        .map_err(|_| AuthenticatorError::InvalidOtpUri)?,
                );
            }
            _ => {}
        }
    }

    let mut secret = secret.ok_or(AuthenticatorError::InvalidOtpUri)?;
    let secret_base32 = std::mem::take(&mut *secret);
    let label_issuer = label
        .split_once(':')
        .map(|(left, _)| left.trim().to_owned())
        .unwrap_or_default();
    let account_label = label
        .split_once(':')
        .map(|(_, right)| right.trim().to_owned())
        .unwrap_or_else(|| label.clone());
    let issuer = issuer.unwrap_or(label_issuer);

    let kind = match host {
        "totp" => OtpKind::Totp { period },
        "hotp" => OtpKind::Hotp {
            counter: counter.ok_or(AuthenticatorError::InvalidOtpUri)?,
        },
        _ => return Err(AuthenticatorError::InvalidOtpUri),
    };

    validate_new_account(NewAccount {
        label: account_label,
        issuer,
        secret_base32,
        algorithm,
        digits,
        kind,
    })
}

fn validate_new_account(mut account: NewAccount) -> Result<NewAccount> {
    account.label = account.label.trim().to_owned();
    account.issuer = account.issuer.trim().to_owned();
    if account.label.is_empty()
        || account.label.len() > MAX_LABEL_LEN
        || account.issuer.len() > MAX_ISSUER_LEN
        || (account.digits != 6 && account.digits != 8)
    {
        account.secret_base32.zeroize();
        return Err(AuthenticatorError::InvalidAccount);
    }
    if let OtpKind::Totp { period } = account.kind {
        if !(15..=120).contains(&period) {
            account.secret_base32.zeroize();
            return Err(AuthenticatorError::InvalidAccount);
        }
    }
    let normalized_secret = match normalize_secret(&account.secret_base32) {
        Ok(secret) => secret,
        Err(error) => {
            account.secret_base32.zeroize();
            return Err(error);
        }
    };
    account.secret_base32.zeroize();
    account.secret_base32 = normalized_secret;
    Ok(account)
}

fn code_view(id: &str, generated: GeneratedCode) -> CodeView {
    CodeView {
        account_id: id.to_owned(),
        code: generated.code,
        valid_for_seconds: generated.valid_for_seconds,
        counter: generated.counter,
    }
}

fn validate_password(password: &[u8]) -> Result<()> {
    if password.len() < 12 {
        return Err(AuthenticatorError::PasswordTooShort);
    }
    Ok(())
}

fn derive_key(password: &[u8], salt: &[u8; SALT_LEN]) -> Result<[u8; KEY_LEN]> {
    let params = Params::new(
        ARGON_MEMORY_KIB,
        ARGON_ITERATIONS,
        ARGON_LANES,
        Some(KEY_LEN),
    )
    .map_err(|_| AuthenticatorError::Crypto)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0_u8; KEY_LEN];
    argon
        .hash_password_into(password, salt, &mut key)
        .map_err(|_| AuthenticatorError::Crypto)?;
    Ok(key)
}

fn header(salt: &[u8; SALT_LEN], nonce: &[u8; NONCE_LEN]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(HEADER_LEN);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    bytes.extend_from_slice(&ARGON_MEMORY_KIB.to_le_bytes());
    bytes.extend_from_slice(&ARGON_ITERATIONS.to_le_bytes());
    bytes.extend_from_slice(&ARGON_LANES.to_le_bytes());
    bytes.extend_from_slice(salt);
    bytes.extend_from_slice(nonce);
    bytes
}

fn parse_header(bytes: &[u8]) -> Result<([u8; SALT_LEN], [u8; NONCE_LEN])> {
    if bytes.len() < HEADER_LEN || &bytes[..4] != MAGIC {
        return Err(AuthenticatorError::InvalidStore);
    }
    let version = u16::from_le_bytes([bytes[4], bytes[5]]);
    if version != FORMAT_VERSION {
        return Err(AuthenticatorError::UnsupportedFormat);
    }
    let memory = read_u32(bytes, 6)?;
    let iterations = read_u32(bytes, 10)?;
    let lanes = read_u32(bytes, 14)?;
    if memory != ARGON_MEMORY_KIB || iterations != ARGON_ITERATIONS || lanes != ARGON_LANES {
        return Err(AuthenticatorError::UnsupportedFormat);
    }

    let mut salt = [0_u8; SALT_LEN];
    salt.copy_from_slice(&bytes[18..18 + SALT_LEN]);
    let mut nonce = [0_u8; NONCE_LEN];
    nonce.copy_from_slice(&bytes[18 + SALT_LEN..HEADER_LEN]);
    Ok((salt, nonce))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or(AuthenticatorError::InvalidStore)?;
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| AuthenticatorError::InvalidStore)?,
    ))
}

fn load_store(path: &Path, password: &[u8]) -> Result<StoreData> {
    validate_password(password)?;
    recover_interrupted_replace(path)?;
    let symlink_metadata =
        fs::symlink_metadata(path).map_err(|_| AuthenticatorError::StoreNotFound)?;
    if symlink_metadata.file_type().is_symlink() {
        return Err(AuthenticatorError::SymlinkNotAllowed);
    }
    let metadata = fs::metadata(path).map_err(|_| AuthenticatorError::StoreNotFound)?;
    if metadata.len() > MAX_STORE_FILE_BYTES {
        return Err(AuthenticatorError::InvalidStore);
    }

    let bytes = fs::read(path).map_err(|_| AuthenticatorError::Io)?;
    if bytes.len() <= HEADER_LEN {
        return Err(AuthenticatorError::InvalidStore);
    }
    let (salt, nonce) = parse_header(&bytes[..HEADER_LEN])?;
    let header_bytes = &bytes[..HEADER_LEN];

    let mut key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AuthenticatorError::Crypto)?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &bytes[HEADER_LEN..],
                aad: header_bytes,
            },
        )
        .map_err(|_| AuthenticatorError::InvalidPassword);
    key.zeroize();

    let plaintext = Zeroizing::new(plaintext?);
    let store: StoreData =
        serde_json::from_slice(&plaintext).map_err(|_| AuthenticatorError::InvalidStore)?;
    if store.version != FORMAT_VERSION || store.accounts.len() > MAX_ACCOUNTS {
        return Err(AuthenticatorError::InvalidStore);
    }
    Ok(store)
}

fn save_store(path: &Path, password: &[u8], store: &StoreData) -> Result<()> {
    validate_password(password)?;
    if store.accounts.len() > MAX_ACCOUNTS {
        return Err(AuthenticatorError::TooManyAccounts);
    }

    let mut plaintext =
        serde_json::to_vec(store).map_err(|_| AuthenticatorError::InvalidStore)?;
    if plaintext.len() as u64 > MAX_PLAINTEXT_STORE_BYTES {
        plaintext.zeroize();
        return Err(AuthenticatorError::InvalidStore);
    }

    let mut salt = [0_u8; SALT_LEN];
    let mut nonce = [0_u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);
    let header_bytes = header(&salt, &nonce);

    let mut key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| AuthenticatorError::Crypto)?;
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &plaintext,
                aad: &header_bytes,
            },
        )
        .map_err(|_| AuthenticatorError::Crypto)?;
    key.zeroize();
    plaintext.zeroize();

    write_atomic(path, &header_bytes, &ciphertext)
}

fn write_atomic(path: &Path, header: &[u8], ciphertext: &[u8]) -> Result<()> {
    if path.exists() {
        let metadata = fs::symlink_metadata(path).map_err(|_| AuthenticatorError::Io)?;
        if metadata.file_type().is_symlink() {
            return Err(AuthenticatorError::SymlinkNotAllowed);
        }
    }
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|_| AuthenticatorError::Io)?;
    let temporary = temporary_path(path)?;

    let backup = path.with_extension("dfauth.bak");
    if backup.exists() {
        fs::remove_file(&backup).map_err(|_| AuthenticatorError::Io)?;
    }

    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| AuthenticatorError::Io)?;
        file.write_all(header).map_err(|_| AuthenticatorError::Io)?;
        file.write_all(ciphertext)
            .map_err(|_| AuthenticatorError::Io)?;
        file.sync_all().map_err(|_| AuthenticatorError::Io)?;

        let had_original = path.exists();
        if had_original {
            fs::rename(path, &backup).map_err(|_| AuthenticatorError::Io)?;
        }

        if fs::rename(&temporary, path).is_err() {
            if had_original {
                let _ = fs::rename(&backup, path);
            }
            return Err(AuthenticatorError::Io);
        }

        if had_original {
            let _ = fs::remove_file(&backup);
        }
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn recover_interrupted_replace(path: &Path) -> Result<()> {
    let backup = path.with_extension("dfauth.bak");
    if backup.exists() {
        let metadata = fs::symlink_metadata(&backup).map_err(|_| AuthenticatorError::Io)?;
        if metadata.file_type().is_symlink() {
            return Err(AuthenticatorError::SymlinkNotAllowed);
        }
    }
    if !path.exists() && backup.exists() {
        fs::rename(&backup, path).map_err(|_| AuthenticatorError::Io)?;
    } else if path.exists() && backup.exists() {
        let _ = fs::remove_file(&backup);
    }
    Ok(())
}

fn temporary_path(path: &Path) -> Result<PathBuf> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(AuthenticatorError::Io)?;
    for _ in 0..16 {
        let mut random = [0_u8; 8];
        OsRng.fill_bytes(&mut random);
        let suffix = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let candidate = parent.join(format!(".{name}.{suffix}.tmp"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(AuthenticatorError::Io)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        add_account, change_password, consume_hotp, create_store, generate_code,
        import_otpauth_uri, list_accounts, reveal_recovery_codes, set_recovery_codes,
    };
    use crate::{AuthenticatorError, NewAccount, OtpAlgorithm, OtpKind};

    const SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";
    const PASSWORD: &[u8] = b"test-password-123";

    #[test]
    fn interrupted_replace_backup_is_recovered() {
        let temp = tempdir().expect("temp");
        let path = temp.path().join("auth.dfauth");
        create_store(&path, PASSWORD).expect("create");
        let backup = path.with_extension("dfauth.bak");
        fs::rename(&path, &backup).expect("simulate interrupted replace");
        assert!(list_accounts(&path, PASSWORD).is_ok());
        assert!(path.is_file());
        assert!(!backup.exists());
    }

    #[test]
    fn encrypted_store_round_trip_hides_secret() {
        let temp = tempdir().expect("temp");
        let path = temp.path().join("auth.dfauth");
        create_store(&path, PASSWORD).expect("create");
        add_account(
            &path,
            PASSWORD,
            NewAccount {
                label: "Example".into(),
                issuer: "DragonForge".into(),
                secret_base32: SECRET.into(),
                algorithm: OtpAlgorithm::Sha1,
                digits: 6,
                kind: OtpKind::Totp { period: 30 },
            },
        )
        .expect("add");

        let raw = fs::read(&path).expect("raw");
        assert!(!raw.windows(SECRET.len()).any(|window| window == SECRET.as_bytes()));
        let accounts = list_accounts(&path, PASSWORD).expect("list");
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].label, "Example");
    }

    #[test]
    fn password_rotation_reencrypts_store() {
        let temp = tempdir().expect("temp");
        let path = temp.path().join("auth.dfauth");
        create_store(&path, PASSWORD).expect("create");
        change_password(&path, PASSWORD, b"rotated-password-456").expect("rotate");
        assert_eq!(
            list_accounts(&path, PASSWORD).expect_err("old password"),
            AuthenticatorError::InvalidPassword
        );
        assert!(list_accounts(&path, b"rotated-password-456").is_ok());
    }

    #[test]
    fn malformed_otpauth_uri_is_rejected() {
        let temp = tempdir().expect("temp");
        let path = temp.path().join("auth.dfauth");
        create_store(&path, PASSWORD).expect("create");
        assert_eq!(
            import_otpauth_uri(&path, PASSWORD, "otpauth://totp/example?issuer=ACME")
                .expect_err("missing secret"),
            AuthenticatorError::InvalidOtpUri
        );
    }

    #[test]
    fn wrong_password_and_tampering_are_rejected() {
        let temp = tempdir().expect("temp");
        let path = temp.path().join("auth.dfauth");
        create_store(&path, PASSWORD).expect("create");
        assert_eq!(
            list_accounts(&path, b"wrong-password").expect_err("wrong"),
            AuthenticatorError::InvalidPassword
        );

        let mut raw = fs::read(&path).expect("read");
        let last = raw.len() - 1;
        raw[last] ^= 0x40;
        fs::write(&path, raw).expect("tamper");
        assert_eq!(
            list_accounts(&path, PASSWORD).expect_err("tampered"),
            AuthenticatorError::InvalidPassword
        );
    }

    #[test]
    fn otpauth_import_and_totp_generation_work() {
        let temp = tempdir().expect("temp");
        let path = temp.path().join("auth.dfauth");
        create_store(&path, PASSWORD).expect("create");
        let account = import_otpauth_uri(
            &path,
            PASSWORD,
            &format!(
                "otpauth://totp/ACME:alice?secret={SECRET}&issuer=ACME&algorithm=SHA1&digits=8&period=30"
            ),
        )
        .expect("import");
        let code = generate_code(&path, PASSWORD, &account.id, 59).expect("code");
        assert_eq!(code.code, "94287082");
    }

    #[test]
    fn hotp_consumption_advances_counter() {
        let temp = tempdir().expect("temp");
        let path = temp.path().join("auth.dfauth");
        create_store(&path, PASSWORD).expect("create");
        let account = add_account(
            &path,
            PASSWORD,
            NewAccount {
                label: "HOTP".into(),
                issuer: String::new(),
                secret_base32: SECRET.into(),
                algorithm: OtpAlgorithm::Sha1,
                digits: 6,
                kind: OtpKind::Hotp { counter: 0 },
            },
        )
        .expect("add");
        let first = consume_hotp(&path, PASSWORD, &account.id).expect("first");
        let second = consume_hotp(&path, PASSWORD, &account.id).expect("second");
        assert_eq!(first.code, "755224");
        assert_eq!(second.code, "287082");
    }

    #[test]
    fn recovery_codes_are_encrypted_and_reveal_only_on_request() {
        let temp = tempdir().expect("temp");
        let path = temp.path().join("auth.dfauth");
        create_store(&path, PASSWORD).expect("create");
        let account = add_account(
            &path,
            PASSWORD,
            NewAccount {
                label: "Recovery".into(),
                issuer: String::new(),
                secret_base32: SECRET.into(),
                algorithm: OtpAlgorithm::Sha1,
                digits: 6,
                kind: OtpKind::Totp { period: 30 },
            },
        )
        .expect("add");
        set_recovery_codes(
            &path,
            PASSWORD,
            &account.id,
            vec!["alpha-beta".into(), "gamma-delta".into()],
        )
        .expect("set");
        let raw = fs::read(&path).expect("read");
        assert!(!raw.windows(10).any(|window| window == b"alpha-beta"));
        assert_eq!(
            reveal_recovery_codes(&path, PASSWORD, &account.id).expect("reveal"),
            vec!["alpha-beta", "gamma-delta"]
        );
    }
}
