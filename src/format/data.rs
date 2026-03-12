use serde_json::Value;
use super::schema::FieldType;

#[derive(Debug, Clone)]
pub enum ForgeValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

impl ForgeValue {
    pub fn from_json(v: &Value) -> Self {
        match v {
            Value::String(s) => ForgeValue::String(s.clone()),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ForgeValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    ForgeValue::Float(f)
                } else {
                    ForgeValue::Null
                }
            }
            Value::Bool(b) => ForgeValue::Boolean(*b),
            _ => ForgeValue::Null,
        }
    }

    pub fn to_json(&self) -> Value {
        match self {
            ForgeValue::String(s)  => Value::String(s.clone()),
            ForgeValue::Integer(i) => Value::Number((*i).into()),
            ForgeValue::Float(f)   => {
                Value::Number(serde_json::Number::from_f64(*f).unwrap_or(0.into()))
            }
            ForgeValue::Boolean(b) => Value::Bool(*b),
            ForgeValue::Null       => Value::Null,
        }
    }

    pub fn field_type(&self) -> FieldType {
        match self {
            ForgeValue::String(_)  => FieldType::String,
            ForgeValue::Integer(_) => FieldType::Integer,
            ForgeValue::Float(_)   => FieldType::Float,
            ForgeValue::Boolean(_) => FieldType::Boolean,
            ForgeValue::Null       => FieldType::Null,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.field_type().to_byte());
        match self {
            ForgeValue::String(s) => {
                let bytes = s.as_bytes();
                buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                buf.extend_from_slice(bytes);
            }
            ForgeValue::Integer(i) => {
                buf.extend_from_slice(&i.to_le_bytes());
            }
            ForgeValue::Float(f) => {
                buf.extend_from_slice(&f.to_le_bytes());
            }
            ForgeValue::Boolean(b) => {
                buf.push(if *b { 1 } else { 0 });
            }
            ForgeValue::Null => {}
        }
        buf
    }

    pub fn from_bytes(buf: &[u8], pos: &mut usize) -> Result<Self, String> {
        if *pos >= buf.len() {
            return Err("Unexpected end of data buffer".to_string());
        }
        let field_type = FieldType::from_byte(buf[*pos])?;
        *pos += 1;

        match field_type {
            FieldType::String => {
                if *pos + 4 > buf.len() {
                    return Err("Buffer too small for string length".to_string());
                }
                let len = u32::from_le_bytes(buf[*pos..*pos + 4].try_into().unwrap()) as usize;
                *pos += 4;
                if *pos + len > buf.len() {
                    return Err("Buffer too small for string data".to_string());
                }
                let s = String::from_utf8(buf[*pos..*pos + len].to_vec())
                    .map_err(|e| e.to_string())?;
                *pos += len;
                Ok(ForgeValue::String(s))
            }
            FieldType::Integer => {
                if *pos + 8 > buf.len() {
                    return Err("Buffer too small for integer".to_string());
                }
                let i = i64::from_le_bytes(buf[*pos..*pos + 8].try_into().unwrap());
                *pos += 8;
                Ok(ForgeValue::Integer(i))
            }
            FieldType::Float => {
                if *pos + 8 > buf.len() {
                    return Err("Buffer too small for float".to_string());
                }
                let f = f64::from_le_bytes(buf[*pos..*pos + 8].try_into().unwrap());
                *pos += 8;
                Ok(ForgeValue::Float(f))
            }
            FieldType::Boolean => {
                if *pos + 1 > buf.len() {
                    return Err("Buffer too small for boolean".to_string());
                }
                let b = buf[*pos] != 0;
                *pos += 1;
                Ok(ForgeValue::Boolean(b))
            }
            FieldType::Null => Ok(ForgeValue::Null),
        }
    }
}

pub type ForgeRow = Vec<(String, ForgeValue)>;

pub fn row_to_bytes(row: &ForgeRow) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(row.len() as u32).to_le_bytes());
    for (name, value) in row {
        let name_bytes = name.as_bytes();
        buf.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(name_bytes);
        buf.extend_from_slice(&value.to_bytes());
    }
    buf
}

pub fn row_from_bytes(buf: &[u8], pos: &mut usize) -> Result<ForgeRow, String> {
    if *pos + 4 > buf.len() {
        return Err("Buffer too small for field count".to_string());
    }
    let field_count = u32::from_le_bytes(buf[*pos..*pos + 4].try_into().unwrap()) as usize;
    *pos += 4;

    let mut row = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        if *pos + 2 > buf.len() {
            return Err("Buffer too small for field name length".to_string());
        }
        let name_len = u16::from_le_bytes(buf[*pos..*pos + 2].try_into().unwrap()) as usize;
        *pos += 2;

        if *pos + name_len > buf.len() {
            return Err("Buffer too small for field name".to_string());
        }
        let name = String::from_utf8(buf[*pos..*pos + name_len].to_vec())
            .map_err(|e| e.to_string())?;
        *pos += name_len;

        let value = ForgeValue::from_bytes(buf, pos)?;
        row.push((name, value));
    }
    Ok(row)
}