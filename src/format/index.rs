pub const ID_SIZE: usize = 36; // UUID v4 string is always 36 chars

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub id: String,
    pub offset: u64,
    pub length: u64,
    pub deleted: bool,
}

#[derive(Debug, Clone)]
pub struct ForgeIndex {
    pub entries: Vec<IndexEntry>,
}

impl ForgeIndex {
    pub fn new() -> Self {
        Self { entries: vec![] }
    }

    pub fn add(&mut self, id: String, offset: u64, length: u64) {
        self.entries.push(IndexEntry { id, offset, length, deleted: false });
    }

    pub fn find(&self, id: &str) -> Option<&IndexEntry> {
        self.entries.iter().find(|e| e.id == id && !e.deleted)
    }

    pub fn mark_deleted(&mut self, id: &str) -> bool {
        match self.entries.iter_mut().find(|e| e.id == id && !e.deleted) {
            Some(e) => { e.deleted = true; true }
            None => false,
        }
    }

    pub fn active_count(&self) -> u64 {
        self.entries.iter().filter(|e| !e.deleted).count() as u64
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(self.entries.len() as u64).to_le_bytes());
        for e in &self.entries {
            let mut id_bytes = [0u8; ID_SIZE];
            let src = e.id.as_bytes();
            id_bytes[..src.len().min(ID_SIZE)].copy_from_slice(&src[..src.len().min(ID_SIZE)]);
            buf.extend_from_slice(&id_bytes);
            buf.extend_from_slice(&e.offset.to_le_bytes());
            buf.extend_from_slice(&e.length.to_le_bytes());
            buf.push(if e.deleted { 1 } else { 0 });
        }
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> Result<(Self, usize), String> {
        if buf.len() < 8 {
            return Err("Index buffer too small".to_string());
        }
        let count = u64::from_le_bytes(buf[0..8].try_into().unwrap()) as usize;
        let mut pos = 8;
        let entry_size = ID_SIZE + 8 + 8 + 1;
        let mut entries = Vec::with_capacity(count);

        for _ in 0..count {
            if pos + entry_size > buf.len() {
                return Err("Unexpected end of index buffer".to_string());
            }
            let id = String::from_utf8(buf[pos..pos + ID_SIZE].to_vec())
                .map_err(|e| e.to_string())?
                .trim_end_matches('\0')
                .to_string();
            pos += ID_SIZE;

            let offset = u64::from_le_bytes(buf[pos..pos + 8].try_into().unwrap());
            pos += 8;
            let length = u64::from_le_bytes(buf[pos..pos + 8].try_into().unwrap());
            pos += 8;
            let deleted = buf[pos] != 0;
            pos += 1;

            entries.push(IndexEntry { id, offset, length, deleted });
        }

        Ok((Self { entries }, pos))
    }
}