use std::{fmt::Display, str};

use async_graphql::{connection::CursorType, *};
use chrono::{DateTime, Utc};

pub struct DateTimeScalar(pub DateTime<Utc>);

#[Scalar]
impl ScalarType for DateTimeScalar {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(str) = &value {
            Ok(DateTimeScalar(
                DateTime::parse_from_rfc3339(str)?.with_timezone(&Utc),
            ))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_rfc3339())
    }
}

impl CursorType for DateTimeScalar {
    type Error = DateTimeCursorDecodeError;

    fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
        let vec = base64::decode(s).map_err(DateTimeCursorDecodeError::Base64)?;
        let s = str::from_utf8(vec.as_slice()).map_err(DateTimeCursorDecodeError::Utf8)?;
        Ok(DateTimeScalar(
            DateTime::parse_from_rfc3339(s)
                .map_err(DateTimeCursorDecodeError::DateTime)?
                .with_timezone(&Utc),
        ))
    }

    fn encode_cursor(&self) -> String {
        base64::encode(self.0.to_rfc3339())
    }
}

#[derive(Debug)]
pub enum DateTimeCursorDecodeError {
    Base64(base64::DecodeError),
    Utf8(str::Utf8Error),
    DateTime(chrono::ParseError),
}

impl Display for DateTimeCursorDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self))
    }
}
