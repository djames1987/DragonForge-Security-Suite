use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component as PathComponent, Path, PathBuf};

use aes_gcm::aead::{Aead, Payload};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand_core::{OsRng, RngCore};
use zeroize::{Zeroize, Zeroizing};

use crate::error::{FileVaultError, Result};
use crate::format::{
    ARGON_ITERATIONS, ARGON_LANES, ARGON_MEMORY_KIB, HEADER_LEN, Header, KEY_LEN,
    MAX_CONTAINER_BYTES, MAX_ENTRIES, MAX_PATH_BYTES, MAX_TOTAL_FILE_BYTES, NONCE_LEN, SALT_LEN,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultEntryInfo {
    pub path: String,
    pub is_directory: bool,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultSummary {
    pub entries: usize,
    pub files: usize,
    pub directories: usize,
    pub total_file_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractSummary {
    pub files: usize,
    pub directories: usize,
    pub total_file_bytes: u64,
}

#[derive(Debug)]
struct DecodedEntry {
    path: String,
    is_directory: bool,
    data: Vec<u8>,
}

impl Drop for DecodedEntry {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

pub fn create_vault(
    output: impl AsRef<Path>,
    sources: &[PathBuf],
    password: &[u8],
) -> Result<VaultSummary> {
    if sources.is_empty() {
        return Err(FileVaultError::InvalidPath);
    }
    validate_password(password)?;

    let output = output.as_ref();
    if output.exists() {
        return Err(FileVaultError::DestinationExists);
    }

    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    let mut total_file_bytes = 0_u64;

    for source in sources {
        collect_source(source, &mut entries, &mut seen, &mut total_file_bytes)?;
    }

    if entries.is_empty() {
        return Err(FileVaultError::SourceNotFound);
    }

    entries.sort_by(|left, right| left.path.cmp(&right.path));
    let summary = summarize(&entries);
    let mut plaintext = encode_entries(&entries)?;

    let mut salt = [0_u8; SALT_LEN];
    let mut nonce = [0_u8; NONCE_LEN];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);
    let header = Header { salt, nonce };
    let header_bytes = header.encode();

    let mut key = derive_key(password, &header.salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| FileVaultError::Crypto)?;
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&header.nonce),
            Payload {
                msg: &plaintext,
                aad: &header_bytes,
            },
        )
        .map_err(|_| FileVaultError::Crypto)?;
    key.zeroize();
    plaintext.zeroize();

    write_container_atomic(output, &header_bytes, &ciphertext)?;
    Ok(summary)
}

pub fn list_vault(container: impl AsRef<Path>, password: &[u8]) -> Result<Vec<VaultEntryInfo>> {
    let entries = open_entries(container.as_ref(), password)?;
    Ok(entries
        .iter()
        .map(|entry| VaultEntryInfo {
            path: entry.path.clone(),
            is_directory: entry.is_directory,
            size: entry.data.len() as u64,
        })
        .collect())
}

pub fn verify_vault(container: impl AsRef<Path>, password: &[u8]) -> Result<VaultSummary> {
    let entries = open_entries(container.as_ref(), password)?;
    Ok(summarize(&entries))
}

pub fn extract_vault(
    container: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    password: &[u8],
) -> Result<ExtractSummary> {
    let destination = destination.as_ref();
    if destination.exists() {
        return Err(FileVaultError::DestinationExists);
    }

    let entries = open_entries(container.as_ref(), password)?;
    let summary = summarize(&entries);
    let temporary = unique_extract_directory(destination)?;

    let extraction = (|| -> Result<()> {
        fs::create_dir(&temporary).map_err(|_| FileVaultError::Io)?;

        for entry in &entries {
            let target = join_archive_path(&temporary, &entry.path)?;
            if entry.is_directory {
                fs::create_dir_all(&target).map_err(|_| FileVaultError::Io)?;
                continue;
            }

            let parent = target.parent().ok_or(FileVaultError::InvalidPath)?;
            fs::create_dir_all(parent).map_err(|_| FileVaultError::Io)?;
            if target.exists() {
                return Err(FileVaultError::DestinationExists);
            }

            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
                .map_err(|_| FileVaultError::Io)?;
            file.write_all(&entry.data)
                .map_err(|_| FileVaultError::Io)?;
            file.sync_all().map_err(|_| FileVaultError::Io)?;
        }

        fs::rename(&temporary, destination).map_err(|_| FileVaultError::Io)?;
        Ok(())
    })();

    if extraction.is_err() {
        let _ = fs::remove_dir_all(&temporary);
    }
    extraction?;

    Ok(ExtractSummary {
        files: summary.files,
        directories: summary.directories,
        total_file_bytes: summary.total_file_bytes,
    })
}

fn collect_source(
    source: &Path,
    entries: &mut Vec<DecodedEntry>,
    seen: &mut HashSet<String>,
    total_file_bytes: &mut u64,
) -> Result<()> {
    if !source.exists() {
        return Err(FileVaultError::SourceNotFound);
    }

    let metadata = fs::symlink_metadata(source).map_err(|_| FileVaultError::Io)?;
    if metadata.file_type().is_symlink() {
        return Err(FileVaultError::SymlinkNotAllowed);
    }

    let root_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(FileVaultError::InvalidPath)?
        .to_owned();
    validate_archive_path(&root_name)?;

    if metadata.is_file() {
        collect_file(source, root_name, entries, seen, total_file_bytes)?;
    } else if metadata.is_dir() {
        collect_directory(source, &root_name, source, entries, seen, total_file_bytes)?;
    } else {
        return Err(FileVaultError::InvalidPath);
    }

    Ok(())
}

fn collect_directory(
    root: &Path,
    root_name: &str,
    directory: &Path,
    entries: &mut Vec<DecodedEntry>,
    seen: &mut HashSet<String>,
    total_file_bytes: &mut u64,
) -> Result<()> {
    let relative = directory
        .strip_prefix(root)
        .map_err(|_| FileVaultError::InvalidPath)?;
    let directory_archive_path = archive_path(root_name, relative)?;
    insert_entry(
        DecodedEntry {
            path: directory_archive_path,
            is_directory: true,
            data: Vec::new(),
        },
        entries,
        seen,
    )?;

    let mut children = fs::read_dir(directory)
        .map_err(|_| FileVaultError::Io)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|_| FileVaultError::Io)?;
    children.sort_by_key(std::fs::DirEntry::path);

    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path).map_err(|_| FileVaultError::Io)?;
        if metadata.file_type().is_symlink() {
            return Err(FileVaultError::SymlinkNotAllowed);
        }

        if metadata.is_dir() {
            collect_directory(root, root_name, &path, entries, seen, total_file_bytes)?;
        } else if metadata.is_file() {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| FileVaultError::InvalidPath)?;
            let archived = archive_path(root_name, relative)?;
            collect_file(&path, archived, entries, seen, total_file_bytes)?;
        } else {
            return Err(FileVaultError::InvalidPath);
        }
    }

    Ok(())
}

fn collect_file(
    source: &Path,
    archive_path: String,
    entries: &mut Vec<DecodedEntry>,
    seen: &mut HashSet<String>,
    total_file_bytes: &mut u64,
) -> Result<()> {
    validate_archive_path(&archive_path)?;
    let metadata = fs::metadata(source).map_err(|_| FileVaultError::Io)?;
    *total_file_bytes = total_file_bytes
        .checked_add(metadata.len())
        .ok_or(FileVaultError::ContainerTooLarge)?;
    if *total_file_bytes > MAX_TOTAL_FILE_BYTES {
        return Err(FileVaultError::ContainerTooLarge);
    }

    let data = fs::read(source).map_err(|_| FileVaultError::Io)?;
    insert_entry(
        DecodedEntry {
            path: archive_path,
            is_directory: false,
            data,
        },
        entries,
        seen,
    )
}

fn insert_entry(
    entry: DecodedEntry,
    entries: &mut Vec<DecodedEntry>,
    seen: &mut HashSet<String>,
) -> Result<()> {
    if entries.len() >= MAX_ENTRIES {
        return Err(FileVaultError::TooManyEntries);
    }
    if !seen.insert(entry.path.clone()) {
        return Err(FileVaultError::InvalidPath);
    }
    entries.push(entry);
    Ok(())
}

fn archive_path(root_name: &str, relative: &Path) -> Result<String> {
    let mut parts = vec![root_name.to_owned()];
    for component in relative.components() {
        match component {
            PathComponent::Normal(value) => {
                parts.push(
                    value
                        .to_str()
                        .ok_or(FileVaultError::InvalidPath)?
                        .to_owned(),
                );
            }
            PathComponent::CurDir => {}
            _ => return Err(FileVaultError::InvalidPath),
        }
    }
    let joined = parts.join("/");
    validate_archive_path(&joined)?;
    Ok(joined)
}

fn encode_entries(entries: &[DecodedEntry]) -> Result<Vec<u8>> {
    let count: u32 = entries
        .len()
        .try_into()
        .map_err(|_| FileVaultError::TooManyEntries)?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&count.to_le_bytes());

    for entry in entries {
        validate_archive_path(&entry.path)?;
        let path = entry.path.as_bytes();
        let path_len: u32 = path
            .len()
            .try_into()
            .map_err(|_| FileVaultError::InvalidPath)?;
        let data_len: u64 = entry
            .data
            .len()
            .try_into()
            .map_err(|_| FileVaultError::ContainerTooLarge)?;

        bytes.push(u8::from(entry.is_directory));
        bytes.extend_from_slice(&path_len.to_le_bytes());
        bytes.extend_from_slice(&data_len.to_le_bytes());
        bytes.extend_from_slice(path);
        bytes.extend_from_slice(&entry.data);
    }

    Ok(bytes)
}

fn decode_entries(bytes: &[u8]) -> Result<Vec<DecodedEntry>> {
    let mut cursor = 0_usize;
    let count = read_u32(bytes, &mut cursor)? as usize;
    if count == 0 || count > MAX_ENTRIES {
        return Err(FileVaultError::InvalidContainer);
    }

    let mut entries = Vec::with_capacity(count);
    let mut seen = HashSet::new();
    let mut total_file_bytes = 0_u64;

    for _ in 0..count {
        let kind = read_u8(bytes, &mut cursor)?;
        if kind > 1 {
            return Err(FileVaultError::InvalidContainer);
        }

        let path_len = read_u32(bytes, &mut cursor)? as usize;
        if path_len == 0 || path_len > MAX_PATH_BYTES {
            return Err(FileVaultError::InvalidContainer);
        }
        let data_len = read_u64(bytes, &mut cursor)?;
        if kind == 1 && data_len != 0 {
            return Err(FileVaultError::InvalidContainer);
        }

        total_file_bytes = total_file_bytes
            .checked_add(data_len)
            .ok_or(FileVaultError::ContainerTooLarge)?;
        if total_file_bytes > MAX_TOTAL_FILE_BYTES {
            return Err(FileVaultError::ContainerTooLarge);
        }

        let path_bytes = read_slice(bytes, &mut cursor, path_len)?;
        let path = std::str::from_utf8(path_bytes)
            .map_err(|_| FileVaultError::InvalidContainer)?
            .to_owned();
        validate_archive_path(&path)?;

        let data_len: usize = data_len
            .try_into()
            .map_err(|_| FileVaultError::ContainerTooLarge)?;
        let data = read_slice(bytes, &mut cursor, data_len)?.to_vec();

        if !seen.insert(path.clone()) {
            return Err(FileVaultError::InvalidContainer);
        }
        entries.push(DecodedEntry {
            path,
            is_directory: kind == 1,
            data,
        });
    }

    if cursor != bytes.len() {
        return Err(FileVaultError::InvalidContainer);
    }

    Ok(entries)
}

fn open_entries(container: &Path, password: &[u8]) -> Result<Vec<DecodedEntry>> {
    validate_password(password)?;

    let metadata = fs::metadata(container).map_err(|_| FileVaultError::Io)?;
    if metadata.len() > MAX_CONTAINER_BYTES {
        return Err(FileVaultError::ContainerTooLarge);
    }

    let bytes = fs::read(container).map_err(|_| FileVaultError::Io)?;
    if bytes.len() <= HEADER_LEN {
        return Err(FileVaultError::InvalidContainer);
    }

    let header = Header::decode(&bytes[..HEADER_LEN])?;
    let header_bytes = &bytes[..HEADER_LEN];
    let ciphertext = &bytes[HEADER_LEN..];

    let mut key = derive_key(password, &header.salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| FileVaultError::Crypto)?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&header.nonce),
            Payload {
                msg: ciphertext,
                aad: header_bytes,
            },
        )
        .map_err(|_| FileVaultError::InvalidPassword);
    key.zeroize();

    let plaintext = Zeroizing::new(plaintext?);
    decode_entries(&plaintext)
}

fn validate_password(password: &[u8]) -> Result<()> {
    if password.len() < 12 {
        return Err(FileVaultError::PasswordTooShort);
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
    .map_err(|_| FileVaultError::Crypto)?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0_u8; KEY_LEN];
    argon
        .hash_password_into(password, salt, &mut key)
        .map_err(|_| FileVaultError::Crypto)?;
    Ok(key)
}

fn write_container_atomic(output: &Path, header: &[u8], ciphertext: &[u8]) -> Result<()> {
    let parent = output.parent().ok_or(FileVaultError::InvalidPath)?;
    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|_| FileVaultError::Io)?;
    }

    let temporary = output.with_extension("dfvault.tmp");
    if temporary.exists() {
        return Err(FileVaultError::DestinationExists);
    }

    let write_result = (|| -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| FileVaultError::Io)?;
        file.write_all(header).map_err(|_| FileVaultError::Io)?;
        file.write_all(ciphertext).map_err(|_| FileVaultError::Io)?;
        file.sync_all().map_err(|_| FileVaultError::Io)?;
        fs::rename(&temporary, output).map_err(|_| FileVaultError::Io)?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result
}

fn unique_extract_directory(destination: &Path) -> Result<PathBuf> {
    let parent = destination.parent().ok_or(FileVaultError::InvalidPath)?;
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(FileVaultError::InvalidPath)?;

    for _ in 0..16 {
        let mut random = [0_u8; 8];
        OsRng.fill_bytes(&mut random);
        let suffix = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let candidate = parent.join(format!(".{name}.dfvtmp-{suffix}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(FileVaultError::Io)
}

fn join_archive_path(root: &Path, archived: &str) -> Result<PathBuf> {
    validate_archive_path(archived)?;
    let mut path = root.to_path_buf();
    for part in archived.split('/') {
        path.push(part);
    }
    Ok(path)
}

fn validate_archive_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\0')
        || path.contains('\\')
        || path.contains(':')
    {
        return Err(FileVaultError::InvalidPath);
    }

    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Err(FileVaultError::InvalidPath);
        }
    }

    Ok(())
}

fn summarize(entries: &[DecodedEntry]) -> VaultSummary {
    let files = entries.iter().filter(|entry| !entry.is_directory).count();
    let directories = entries.len() - files;
    let total_file_bytes = entries
        .iter()
        .filter(|entry| !entry.is_directory)
        .map(|entry| entry.data.len() as u64)
        .sum();

    VaultSummary {
        entries: entries.len(),
        files,
        directories,
        total_file_bytes,
    }
}

fn read_u8(bytes: &[u8], cursor: &mut usize) -> Result<u8> {
    let value = *bytes.get(*cursor).ok_or(FileVaultError::InvalidContainer)?;
    *cursor += 1;
    Ok(value)
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32> {
    let value = read_slice(bytes, cursor, 4)?;
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| FileVaultError::InvalidContainer)?,
    ))
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64> {
    let value = read_slice(bytes, cursor, 8)?;
    Ok(u64::from_le_bytes(
        value
            .try_into()
            .map_err(|_| FileVaultError::InvalidContainer)?,
    ))
}

fn read_slice<'a>(bytes: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8]> {
    let end = cursor
        .checked_add(len)
        .ok_or(FileVaultError::InvalidContainer)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(FileVaultError::InvalidContainer)?;
    *cursor = end;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        DecodedEntry, create_vault, decode_entries, encode_entries, extract_vault, list_vault,
        verify_vault,
    };
    use crate::FileVaultError;

    #[test]
    fn file_and_folder_round_trip_encrypts_names_and_contents() {
        let temp = tempdir().expect("temp");
        let source = temp.path().join("private");
        fs::create_dir_all(source.join("nested")).expect("source dirs");
        fs::create_dir_all(source.join("empty")).expect("empty dir");
        fs::write(source.join("secret.txt"), b"dragon fire").expect("file");
        fs::write(source.join("nested").join("note.bin"), [1_u8, 2, 3, 4]).expect("file");

        let vault = temp.path().join("archive.dfvault");
        let summary = create_vault(&vault, std::slice::from_ref(&source), b"test-password-123")
            .expect("create");
        assert_eq!(summary.files, 2);

        let raw = fs::read(&vault).expect("vault bytes");
        assert!(
            !raw.windows(b"secret.txt".len())
                .any(|window| window == b"secret.txt")
        );
        assert!(
            !raw.windows(b"dragon fire".len())
                .any(|window| window == b"dragon fire")
        );

        let listing = list_vault(&vault, b"test-password-123").expect("list");
        assert!(
            listing
                .iter()
                .any(|entry| entry.path == "private/secret.txt")
        );
        assert!(listing.iter().any(|entry| entry.path == "private/empty"));

        let extracted = temp.path().join("restored");
        let result = extract_vault(&vault, &extracted, b"test-password-123").expect("extract");
        assert_eq!(result.files, 2);
        assert_eq!(
            fs::read(extracted.join("private").join("secret.txt")).expect("restored"),
            b"dragon fire"
        );
    }

    #[test]
    fn short_passwords_are_rejected_by_native_engine() {
        let temp = tempdir().expect("temp");
        let source = temp.path().join("a.txt");
        fs::write(&source, b"secret").expect("file");
        let vault = temp.path().join("a.dfvault");
        assert_eq!(
            create_vault(&vault, &[source], b"short").expect_err("short password"),
            FileVaultError::PasswordTooShort
        );
    }

    #[test]
    fn wrong_password_is_rejected() {
        let temp = tempdir().expect("temp");
        let source = temp.path().join("a.txt");
        fs::write(&source, b"secret").expect("file");
        let vault = temp.path().join("a.dfvault");
        create_vault(&vault, &[source], b"correct-password").expect("create");
        assert_eq!(
            verify_vault(&vault, b"wrong-password").expect_err("wrong password"),
            FileVaultError::InvalidPassword
        );
    }

    #[test]
    fn ciphertext_tampering_is_detected() {
        let temp = tempdir().expect("temp");
        let source = temp.path().join("a.txt");
        fs::write(&source, b"secret").expect("file");
        let vault = temp.path().join("a.dfvault");
        create_vault(&vault, &[source], b"correct-password").expect("create");

        let mut bytes = fs::read(&vault).expect("vault");
        let last = bytes.len() - 1;
        bytes[last] ^= 0x80;
        fs::write(&vault, bytes).expect("tamper");

        assert_eq!(
            verify_vault(&vault, b"correct-password").expect_err("tampered"),
            FileVaultError::InvalidPassword
        );
    }

    #[test]
    fn existing_destination_is_never_overwritten() {
        let temp = tempdir().expect("temp");
        let source = temp.path().join("a.txt");
        fs::write(&source, b"secret").expect("file");
        let vault = temp.path().join("a.dfvault");
        create_vault(&vault, &[source], b"correct-password").expect("create");

        let destination = temp.path().join("existing");
        fs::create_dir(&destination).expect("existing");
        assert_eq!(
            extract_vault(&vault, &destination, b"correct-password").expect_err("must reject"),
            FileVaultError::DestinationExists
        );
    }

    #[test]
    fn symlink_sources_are_rejected_when_supported() {
        let temp = tempdir().expect("temp");
        let source = temp.path().join("a.txt");
        fs::write(&source, b"secret").expect("file");
        let link = temp.path().join("link.txt");

        #[cfg(unix)]
        std::os::unix::fs::symlink(&source, &link).expect("symlink");
        #[cfg(windows)]
        if std::os::windows::fs::symlink_file(&source, &link).is_err() {
            return;
        }

        let vault = temp.path().join("a.dfvault");
        assert_eq!(
            create_vault(&vault, &[link], b"correct-password").expect_err("symlink rejected"),
            FileVaultError::SymlinkNotAllowed
        );
    }
    #[test]
    fn decoded_entries_reject_duplicate_and_malformed_paths() {
        let duplicate = vec![
            DecodedEntry {
                path: "safe/file.txt".to_owned(),
                is_directory: false,
                data: vec![1],
            },
            DecodedEntry {
                path: "safe/file.txt".to_owned(),
                is_directory: false,
                data: vec![2],
            },
        ];
        let encoded = encode_entries(&duplicate).expect("encode duplicate test payload");
        assert!(decode_entries(&encoded).is_err());

        let mut malformed = Vec::new();
        malformed.extend_from_slice(&1_u32.to_le_bytes());
        malformed.push(0);
        malformed.extend_from_slice(&8_u32.to_le_bytes());
        malformed.extend_from_slice(&1_u64.to_le_bytes());
        malformed.extend_from_slice(b"../x.txt");
        malformed.push(7);
        assert!(decode_entries(&malformed).is_err());
    }

    #[test]
    fn decoded_entries_reject_truncated_payload() {
        assert!(decode_entries(&[1, 0, 0]).is_err());
    }

}
