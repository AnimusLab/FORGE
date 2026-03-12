pub mod data;
pub mod header;
pub mod index;
pub mod schema;

pub use data::{row_from_bytes, row_to_bytes, ForgeRow, ForgeValue};
pub use header::{ForgeHeader, HEADER_SIZE};
pub use index::ForgeIndex;
pub use schema::ForgeSchema;

use serde_json::{Map, Value};

pub struct ForgeFile {
    pub header: ForgeHeader,
    pub schema: ForgeSchema,
    pub index: ForgeIndex,
    pub data: Vec<u8>,
}

impl ForgeFile {
    pub fn new() -> Self {
        Self {
            header: ForgeHeader::new(),
            schema: ForgeSchema::new(),
            index: ForgeIndex::new(),
            data: Vec::new(),
        }
    }

    pub fn insert(&mut self, id: String, json: &Map<String, Value>) -> Result<(), String> {
        let mut row: ForgeRow = Vec::new();
        for (k, v) in json {
            let val = ForgeValue::from_json(v);
            self.schema.add_field(k.clone(), val.field_type());
            row.push((k.clone(), val));
        }

        let row_bytes = row_to_bytes(&row);
        let offset = self.data.len() as u64;
        let length = row_bytes.len() as u64;

        self.data.extend_from_slice(&row_bytes);
        self.index.add(id, offset, length);
        self.header.row_count = self.index.active_count();
        self.header.schema_hash = self.schema.hash();

        Ok(())
    }

    pub fn get_all(&self) -> Result<Vec<Value>, String> {
        let mut results = Vec::new();
        for entry in &self.index.entries {
            if entry.deleted {
                continue;
            }
            let mut pos = entry.offset as usize;
            let row = row_from_bytes(&self.data, &mut pos)?;
            let mut obj = Map::new();
            obj.insert("_id".to_string(), Value::String(entry.id.clone()));
            for (k, v) in row {
                obj.insert(k, v.to_json());
            }
            results.push(Value::Object(obj));
        }
        Ok(results)
    }

    pub fn get_one(&self, id: &str) -> Result<Option<Value>, String> {
        let entry = match self.index.find(id) {
            Some(e) => e.clone(),
            None => return Ok(None),
        };
        let mut pos = entry.offset as usize;
        let row = row_from_bytes(&self.data, &mut pos)?;
        let mut obj = Map::new();
        obj.insert("_id".to_string(), Value::String(entry.id.clone()));
        for (k, v) in row {
            obj.insert(k, v.to_json());
        }
        Ok(Some(Value::Object(obj)))
    }

    pub fn update(&mut self, id: &str, json: &Map<String, Value>) -> Result<bool, String> {
        let existing = match self.get_one(id)? {
            Some(v) => v,
            None => return Ok(false),
        };

        self.index.mark_deleted(id);

        let mut merged = match existing {
            Value::Object(m) => m,
            _ => Map::new(),
        };
        merged.remove("_id");
        for (k, v) in json {
            merged.insert(k.clone(), v.clone());
        }

        self.insert(id.to_string(), &merged)?;
        self.header.row_count = self.index.active_count();
        Ok(true)
    }

    pub fn delete(&mut self, id: &str) -> bool {
        let deleted = self.index.mark_deleted(id);
        if deleted {
            self.header.row_count = self.index.active_count();
        }
        deleted
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let schema_bytes = self.schema.to_bytes();
        let index_bytes = self.index.to_bytes();

        let mut buf = Vec::new();
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&(schema_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(&schema_bytes);
        buf.extend_from_slice(&(index_bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(&index_bytes);
        buf.extend_from_slice(&self.data);
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < HEADER_SIZE {
            return Err("File too small to be a valid .forge file".to_string());
        }

        let header_arr: &[u8; HEADER_SIZE] = buf[0..HEADER_SIZE].try_into().unwrap();
        let header = ForgeHeader::from_bytes(header_arr)?;
        let mut pos = HEADER_SIZE;

        if pos + 4 > buf.len() {
            return Err("Missing schema length".to_string());
        }
        let schema_len = u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let (schema, _) = ForgeSchema::from_bytes(&buf[pos..pos + schema_len])?;
        pos += schema_len;

        if pos + 4 > buf.len() {
            return Err("Missing index length".to_string());
        }
        let index_len = u32::from_le_bytes(buf[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        let (index, _) = ForgeIndex::from_bytes(&buf[pos..pos + index_len])?;
        pos += index_len;

        let data = buf[pos..].to_vec();

        Ok(Self { header, schema, index, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_insert_and_get() {
        let mut f = ForgeFile::new();
        let obj = json!({"name": "John", "age": 25});
        f.insert("id-001".to_string(), obj.as_object().unwrap()).unwrap();

        let records = f.get_all().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["name"], "John");
        assert_eq!(records[0]["age"], 25);
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut f = ForgeFile::new();
        let obj = json!({"name": "Jane", "score": 99.5, "active": true});
        f.insert("id-001".to_string(), obj.as_object().unwrap()).unwrap();

        let bytes = f.to_bytes();
        let f2 = ForgeFile::from_bytes(&bytes).unwrap();
        let records = f2.get_all().unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["name"], "Jane");
    }

    #[test]
    fn test_delete() {
        let mut f = ForgeFile::new();
        let obj = json!({"name": "John"});
        f.insert("id-001".to_string(), obj.as_object().unwrap()).unwrap();
        f.delete("id-001");
        assert_eq!(f.get_all().unwrap().len(), 0);
    }

    #[test]
    fn test_update() {
        let mut f = ForgeFile::new();
        let obj = json!({"name": "John", "age": 25});
        f.insert("id-001".to_string(), obj.as_object().unwrap()).unwrap();

        let update = json!({"age": 30});
        f.update("id-001", update.as_object().unwrap()).unwrap();

        let record = f.get_one("id-001").unwrap().unwrap();
        assert_eq!(record["age"], 30);
        assert_eq!(record["name"], "John");
    }
}