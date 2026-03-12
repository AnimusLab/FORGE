#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Null,
}

impl FieldType {
    pub fn to_byte(&self) -> u8 {
        match self {
            FieldType::String  => 0x01,
            FieldType::Integer => 0x02,
            FieldType::Float   => 0x03,
            FieldType::Boolean => 0x04,
            FieldType::Null    => 0x05,
        }
    }

    pub fn from_byte(b: u8) -> Result<Self, String> {
        match b {
            0x01 => Ok(FieldType::String),
            0x02 => Ok(FieldType::Integer),
            0x03 => Ok(FieldType::Float),
            0x04 => Ok(FieldType::Boolean),
            0x05 => Ok(FieldType::Null),
            _    => Err(format!("Unknown field type byte: 0x{:02x}", b)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub field_type: FieldType,
}

#[derive(Debug, Clone)]
pub struct ForgeSchema {
    pub fields: Vec<SchemaField>,
}

impl ForgeSchema {
    pub fn new() -> Self {
        Self { fields: vec![] }
    }

    pub fn add_field(&mut self, name: String, field_type: FieldType) {
        if !self.fields.iter().any(|f| f.name == name) {
            self.fields.push(SchemaField { name, field_type });
        }
    }

    // FNV-1a hash of field names + types for integrity checking
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for f in &self.fields {
            for b in f.name.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            h ^= f.field_type.to_byte() as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(self.fields.len() as u32).to_le_bytes());
        for f in &self.fields {
            let name_bytes = f.name.as_bytes();
            buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(name_bytes);
            buf.push(f.field_type.to_byte());
        }
        buf
    }

    pub fn from_bytes(buf: &[u8]) -> Result<(Self, usize), String> {
        if buf.len() < 4 {
            return Err("Schema buffer too small".to_string());
        }
        let field_count = u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize;
        let mut pos = 4;
        let mut fields = Vec::with_capacity(field_count);

        for _ in 0..field_count {
            if pos + 2 > buf.len() {
                return Err("Unexpected end of schema buffer".to_string());
            }
            let name_len = u16::from_le_bytes(buf[pos..pos + 2].try_into().unwrap()) as usize;
            pos += 2;

            if pos + name_len + 1 > buf.len() {
                return Err("Unexpected end of schema buffer".to_string());
            }
            let name = String::from_utf8(buf[pos..pos + name_len].to_vec())
                .map_err(|e| e.to_string())?;
            pos += name_len;

            let field_type = FieldType::from_byte(buf[pos])?;
            pos += 1;

            fields.push(SchemaField { name, field_type });
        }

        Ok((Self { fields }, pos))
    }
}