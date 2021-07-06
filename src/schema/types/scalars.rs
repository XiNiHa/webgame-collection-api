use async_graphql::*;
use chrono::{DateTime, Utc};
use sqlx::types::Uuid;

pub struct UuidScalar(pub Uuid);

#[Scalar]
impl ScalarType for UuidScalar {
    fn parse(value: Value) -> InputValueResult<Self> {
        if let Value::String(str) = &value {
            Ok(UuidScalar(Uuid::parse_str(str)?))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

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
