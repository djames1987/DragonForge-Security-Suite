pub const MAGIC: &[u8; 8] = b"DFSHARE!";
pub const FORMAT_VERSION: u16 = 1;
pub const SHARE_EXTENSION: &str = "dfshare";

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const TAG_LEN: usize = 16;
pub const HEADER_LEN: usize = MAGIC.len() + 2 + SALT_LEN + NONCE_LEN + 8;

pub const MAX_ATTACHMENTS: usize = 2_048;
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_SECRET_BYTES: usize = 64 * 1024;
pub const MAX_PACKAGE_BYTES: u64 = 384 * 1024 * 1024;
pub const MAX_PATH_CHARS: usize = 1_024;
pub const MAX_LABEL_CHARS: usize = 200;
