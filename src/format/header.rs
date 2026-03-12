use std::time::{SystemTime, UNIX_EPOCH};

pub const MAGIC: &[u8; 8] = b"FORGE001";
pub const VERSION: u8 = 1;
pub const HEADER_SIZE: usize = 64;

#[derive(Debug, Clone)]
pub struct ForgeHeader {
    pub version: u8,
    pub created_at: i64,
    pub row_count: u64,
    pub schema_hash: u64,
}

impl ForgeHeader {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Self {
            version: VERSION,
            created_at: now,
            row_count: 0,
            schema_hash: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..8].copy_from_slice(MAGIC);
        buf[8] = self.version;
        buf[9..17].copy_from_slice(&self.created_at.to_le_bytes());
        buf[17..25].copy_from_slice(&self.row_count.to_le_bytes());
        buf[25..33].copy_from_slice(&self.schema_hash.to_le_bytes());
        // bytes 33..64 reserved for future use
        buf
    }

    pub fn from_bytes(buf: &[u8; HEADER_SIZE]) -> Result<Self, String> {
        if &buf[0..8] != MAGIC {
            return Err("Invalid FORGE file: bad magic bytes".to_string());
        }
        let version = buf[8];
        let created_at = i64::from_le_bytes(buf[9..17].try_into().unwrap());
        let row_count = u64::from_le_bytes(buf[17..25].try_into().unwrap());
        let schema_hash = u64::from_le_bytes(buf[25..33].try_into().unwrap());
        Ok(Self { version, created_at, row_count, schema_hash })
    }
}