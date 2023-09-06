pub mod abi;

use abi::{ command_request::RequestData, * };
use http::StatusCode;

use crate::KvError;

impl CommandRequest {
    /// Create HSET command
    pub fn new_hset(table: impl Into<String>, key: impl Into<String>, value: Value) -> Self {
        Self {
            request_data: Some(
                RequestData::Hset(Hset {
                    table: table.into(),
                    pair: Some(Kvpair::new(key, value)),
                })
            ),
        }
    }

    /// Create HGET command
    pub fn new_hget(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(
                RequestData::Hget(Hget {
                    table: table.into(),
                    key: key.into(),
                })
            ),
        }
    }

    /// Create HGETALL command
    pub fn new_hgetall(table: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hgetall(Hgetall { table: table.into() })),
        }
    }
}

impl Kvpair {
    /// create a new kv pair
    pub fn new(key: impl Into<String>, value: Value) -> Self {
        Self {
            key: key.into(),
            value: Some(value),
        }
    }
}

/// transform String to Value
impl From<String> for Value {
    fn from(s: String) -> Self {
        Self {
            value: Some(value::Value::String(s)),
        }
    }
}

/// transform &str to Value
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

/// Tansform KvError to CommandResponse
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
