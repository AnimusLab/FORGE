use std::collections::HashMap;
use crate::engine::idempotency::IdempotencyStore;
use crate::engine::wal::{Wal, WalEntry, WalOp};
use crate::format::ForgeFile;
use serde_json::Value;

pub struct AppState {
    pub collections: HashMap<String, ForgeFile>,
    pub wal: Wal,
    pub idempotency: IdempotencyStore,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
            wal: Wal::new("forge.wal"),
            idempotency: IdempotencyStore::new(),
        }
    }

    pub fn replay_wal(&mut self) {
        let pending = match self.wal.pending_entries() {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("[WAL] Failed to read WAL on startup: {}", e);
                return;
            }
        };

        if pending.is_empty() {
            tracing::info!("[WAL] No pending entries to replay");
            return;
        }

        tracing::info!("[WAL] Replaying {} pending entries", pending.len());

        for entry in pending {
            match entry.op {
                WalOp::Insert => {
                    if let Some(data) = &entry.data {
                        if let Some(obj) = data.as_object() {
                            let file = self.collections
                                .entry(entry.collection.clone())
                                .or_insert_with(ForgeFile::new);
                            match file.insert(entry.record_id.clone(), obj) {
                                Ok(_) => {
                                    let _ = self.wal.mark_committed(&entry.entry_id);
                                    tracing::info!("[WAL] Replayed INSERT id='{}'", entry.record_id);
                                }
                                Err(e) => tracing::warn!("[WAL] Replay INSERT failed: {}", e),
                            }
                        }
                    }
                }
                WalOp::Update => {
                    if let Some(data) = &entry.data {
                        if let Some(obj) = data.as_object() {
                            if let Some(file) = self.collections.get_mut(&entry.collection) {
                                match file.update(&entry.record_id, obj) {
                                    Ok(_) => {
                                        let _ = self.wal.mark_committed(&entry.entry_id);
                                        tracing::info!("[WAL] Replayed UPDATE id='{}'", entry.record_id);
                                    }
                                    Err(e) => tracing::warn!("[WAL] Replay UPDATE failed: {}", e),
                                }
                            }
                        }
                    }
                }
                WalOp::Delete => {
                    if let Some(file) = self.collections.get_mut(&entry.collection) {
                        file.delete(&entry.record_id);
                        let _ = self.wal.mark_committed(&entry.entry_id);
                        tracing::info!("[WAL] Replayed DELETE id='{}'", entry.record_id);
                    }
                }
            }
        }
    }

    pub fn get_or_create(&mut self, name: &str) -> &mut ForgeFile {
        self.collections
            .entry(name.to_string())
            .or_insert_with(ForgeFile::new)
    }

    pub fn get(&self, name: &str) -> Option<&ForgeFile> {
        self.collections.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut ForgeFile> {
        self.collections.get_mut(name)
    }

    pub fn collection_names(&self) -> Vec<String> {
        self.collections.keys().cloned().collect()
    }
}