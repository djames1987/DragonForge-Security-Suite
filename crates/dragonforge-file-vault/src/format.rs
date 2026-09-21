use crate::error::{FileVaultError, Result};

pub const FILE_VAULT_EXTENSION: &str = "dfvault";
pub const FORMAT_VERSION: u16 = 1;

pub(crate) const MAGIC: &[u8; 4] = b"DFV1";
pub(crate) const SALT_LEN: usize = 16;
pub(crate) const NONCE_LEN: usize = 12;
pub(crate) const KEY_LEN: usize = 32;
pub(crate) const HEADER_LEN: usize = 4 + 2 + 4 + 4 + 4 + SALT_LEN + NONCE_LEN;

pub(crate) const ARGON_MEMORY_KIB: u32 = 65_536;
pub(crate) const ARGON_ITERATIONS: u32 = 3;
pub(crate) const ARGON_LANES: u32 = 1;

pub(crate) const MAX_ENTRIES: usize = 10_000;
pub(crate) const MAX_PATH_BYTES: usize = 4_096;
pub(crate) const MAX_TOTAL_FILE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Header {
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
}

impl Header {
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_LEN);
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&ARGON_MEMORY_KIB.to_le_bytes());
        bytes.extend_from_slice(&ARGON_ITERATIONS.to_le_bytes());
        bytes.extend_from_slice(&ARGON_LANES.to_le_bytes());
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.nonce);
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HEADER_LEN || &bytes[..4] != MAGIC {
            return Err(FileVaultError::InvalidContainer);
        }

        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != FORMAT_VERSION {
            return Err(FileVaultError::UnsupportedFormat);
        }

        let memory = read_u32(bytes, 6)?;
        let iterations = read_u32(bytes, 10)?;
        let lanes = read_u32(bytes, 14)?;
        if memory != ARGON_MEMORY_KIB || iterations != ARGON_ITERATIONS || lanes != ARGON_LANES {
            return Err(FileVaultError::UnsupportedFormat);
        }

        let mut salt = [0_u8; SALT_LEN];
        salt.copy_from_slice(&bytes[18..18 + SALT_LEN]);
        let mut nonce = [0_u8; NONCE_LEN];
        nonce.copy_from_slice(&bytes[18 + SALT_LEN..HEADER_LEN]);

        Ok(Self { salt, nonce })
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let value = bytes
        .get(offset..offset + 4)
        .ok_or(FileVaultError::InvalidContainer)?;
    Ok(u32::from_le_bytes(
        value
            .try_into()
            .map_err(|_| FileVaultError::InvalidContainer)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{Header, NONCE_LEN, SALT_LEN};

    #[test]
    fn header_round_trip() {
        let header = Header {
            salt: [7; SALT_LEN],
            nonce: [9; NONCE_LEN],
        };
        assert_eq!(Header::decode(&header.encode()).expect("header"), header);
    }

    #[test]
    fn header_rejects_bad_magic() {
        let mut bytes = Header {
            salt: [7; SALT_LEN],
            nonce: [9; NONCE_LEN],
        }
        .encode();
        bytes[0] = b'X';
        assert!(Header::decode(&bytes).is_err());
    }
}
