pub mod abi;

use abi::{command_request::RequestData, *};
use bytes::Bytes;
use http::StatusCode;
use prost::Message;

use crate::KvError;

impl CommandRequest {
    /// Create HSET command
    pub fn new_hset(table: impl Into<String>, key: impl Into<String>, value: Value) -> Self {
        Self {
            request_data: Some(RequestData::Hset(Hset {
                table: table.into(),
                pair: Some(Kvpair::new(key, value)),
            })),
        }
    }

    /// Create HGET command
    pub fn new_hget(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hget(Hget {
                table: table.into(),
                key: key.into(),
            })),
        }
    }

    /// Create HGETALL command
    pub fn new_hgetall(table: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hgetall(Hgetall {
                table: table.into(),
            })),
        }
    }
}

impl Kvpair {
    /// Create a new kv pair
    pub fn new(key: impl Into<String>, value: Value) -> Self {
        Self {
            key: key.into(),
            value: Some(value),
        }
    }
}

/// Transform String to Value
impl From<String> for Value {
    fn from(s: String) -> Self {
        Self {
            value: Some(value::Value::String(s)),
        }
    }
}

/// Transform &str to Value
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self {
            value: Some(value::Value::String(s.into())),
        }
    }
}

/// Transform i64 to Value
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self {
            value: Some(value::Value::Integer(value)),
        }
    }
}

/// Transform value to CommandResponse
impl From<Value> for CommandResponse {
    fn from(value: Value) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            values: vec![value],
            ..Default::default()
        }
    }
}

/// Transform Vec<Kvpair> to CommandResponse
impl From<Vec<Kvpair>> for CommandResponse {
    fn from(value: Vec<Kvpair>) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            pairs: value,
            ..Default::default()
        }
    }
}

/// Transform KvError to CommandResponse
impl From<KvError> for CommandResponse {
    fn from(e: KvError) -> Self {
        let mut result = Self {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as _,
            message: e.to_string(),
            values: vec![],
            pairs: vec![],
        };

        match e {
            KvError::NotFound(_, _) => {
                result.status = StatusCode::NOT_FOUND.as_u16() as _;
            }
            KvError::InvalidCommand(_) => {
                result.status = StatusCode::BAD_REQUEST.as_u16() as _;
            }
            _ => {}
        }

        result
    }
}

impl From<Vec<Value>> for CommandResponse {
    fn from(value: Vec<Value>) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            values: value,
            ..Default::default()
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self {
            value: Some(value::Value::Bool(value)),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self {
            value: Some(value::Value::Float(value)),
        }
    }
}

impl TryFrom<Value> for i64 {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Integer(i)) => Ok(i),
            _ => Err(KvError::ConvertError(v, "Integer")),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Float(f)) => Ok(f),
            _ => Err(KvError::ConvertError(v, "Float")),
        }
    }
}

impl TryFrom<Value> for Bytes {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Binary(b)) => Ok(b),
            _ => Err(KvError::ConvertError(v, "Binary")),
        }
    }
}

impl TryFrom<Value> for bool {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Bool(b)) => Ok(b),
            _ => Err(KvError::ConvertError(v, "Boolean")),
        }
    }
}

impl TryFrom<Value> for Vec<u8> {
    type Error = KvError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        let mut buf = Vec::with_capacity(v.encoded_len());
        v.encode(&mut buf)?;
        Ok(buf)
    }
}

impl TryFrom<&[u8]> for Value {
    type Error = KvError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let msg = Value::decode(data)?;
        Ok(msg)
    }
}
