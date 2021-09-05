use std::{fmt, num::NonZeroU32};

use config::{Config, ConfigError, Environment};
use serde::{de, Deserialize};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub pbkdf2_salt_size: usize,
    pub pbkdf2_iterations: NonZeroU32,
    #[serde(deserialize_with = "deserialize_base64_string")]
    pub jwt_secret: Vec<u8>,
    pub refresh_token_size: usize,
    pub redis: deadpool_redis::Config,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = Config::new();
        cfg.merge(Environment::new().separator("__"))?;
        cfg.try_into()
    }
}

fn deserialize_base64_string<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct Base64StringVisitor;

    impl<'de> de::Visitor<'de> for Base64StringVisitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a base64 encoded string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            base64::decode(v).map_err(E::custom)
        }
    }

    deserializer.deserialize_any(Base64StringVisitor)
}

lazy_static::lazy_static! {
    pub static ref CONFIG: AppConfig = {
        dotenv::dotenv().ok();

        AppConfig::from_env().unwrap()
    };
}
