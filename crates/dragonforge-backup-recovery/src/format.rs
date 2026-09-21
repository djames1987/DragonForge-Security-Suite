pub const MAGIC: &[u8; 8] = b"DFBACKUP";
pub const FORMAT_VERSION: u16 = 1;
pub const BACKUP_EXTENSION: &str = "dfbackup";

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;
pub const HEADER_LEN: usize = MAGIC.len() + 2 + SALT_LEN + NONCE_LEN + 8;

pub const MAX_ENTRIES: usize = 4_096;
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_ARCHIVE_BYTES: u64 = 768 * 1024 * 1024;
pub const MAX_PATH_CHARS: usize = 1_024;
