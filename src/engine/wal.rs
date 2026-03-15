use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WalOp {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WalStatus {
    Pending,
    Committed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub entry_id: String,
    pub op: WalOp,
    pub collection: String,
    pub record_id: String,
    pub data: Option<Value>,
    pub status: WalStatus,
    pub timestamp: i64,
}

impl WalEntry {
    pub fn new(
        entry_id: String,
        op: WalOp,
        collection: String,
        record_id: String,
        data: Option<Value>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        Self {
            entry_id,
            op,
            collection,
            record_id,
            data,
            status: WalStatus::Pending,
            timestamp,
        }
    }
}

pub struct Wal {
    path: String,
}

impl Wal {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string() }
    }

    pub fn append(&self, entry: &WalEntry) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| e.to_string())?;

        let json = serde_json::to_vec(entry).map_err(|e| e.to_string())?;
        let len = json.len() as u32;
        file.write_all(&len.to_le_bytes()).map_err(|e| e.to_string())?;
        file.write_all(&json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn read_all(&self) -> Result<Vec<WalEntry>, String> {
        let mut file = match File::open(&self.path) {
            Ok(f) => f,
            Err(_) => return Ok(vec![]),
        };

        let mut entries = Vec::new();
        loop {
            let mut len_buf = [0u8; 4];
            match file.read_exact(&mut len_buf) {
                Ok(_) => {}
                Err(_) => break,
            }
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut data = vec![0u8; len];
            file.read_exact(&mut data).map_err(|e| e.to_string())?;
            let entry: WalEntry = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
            entries.push(entry);
        }
        Ok(entries)
    }

    pub fn mark_committed(&self, entry_id: &str) -> Result<(), String> {
        let mut entries = self.read_all()?;
        for e in &mut entries {
            if e.entry_id == entry_id {
                e.status = WalStatus::Committed;
            }
        }
        self.rewrite(&entries)
    }

    pub fn pending_entries(&self) -> Result<Vec<WalEntry>, String> {
        Ok(self.read_all()?
            .into_iter()
            .filter(|e| e.status == WalStatus::Pending)
            .collect())
    }

    fn rewrite(&self, entries: &[WalEntry]) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| e.to_string())?;

        for entry in entries {
            let json = serde_json::to_vec(entry).map_err(|e| e.to_string())?;
            let len = json.len() as u32;
            file.write_all(&len.to_le_bytes()).map_err(|e| e.to_string())?;
            file.write_all(&json).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}