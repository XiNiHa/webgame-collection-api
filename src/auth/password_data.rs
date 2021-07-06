use std::num::NonZeroU32;

use ring::{
    digest,
    pbkdf2::{self, PBKDF2_HMAC_SHA512},
    rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize, Deserialize)]
pub struct PasswordData {
    hash: String,
    salt: String,
    iterations: NonZeroU32,
}

#[derive(Debug)]
pub enum PasswordEncryptionError {
    SaltCreationFailed,
}

impl Error for PasswordEncryptionError {
    fn message(&self) -> String {
        match self {
            PasswordEncryptionError::SaltCreationFailed => "Failed to create salt",
        }
        .to_owned()
    }

    fn code(&self) -> String {
        format!("PasswordEncryptionError::{:?}", self)
    }
}

impl PasswordData {
    pub fn new(
        password: &str,
        salt_size: usize,
        iterations: NonZeroU32,
    ) -> Result<PasswordData, PasswordEncryptionError> {
        let salt = PasswordData::salt(salt_size)
            .map_err(|_| PasswordEncryptionError::SaltCreationFailed)?;
        let mut hash = [0; digest::SHA512_OUTPUT_LEN];
        pbkdf2::derive(
            PBKDF2_HMAC_SHA512,
            iterations,
            &salt,
            password.as_bytes(),
            &mut hash,
        );

        Ok(PasswordData {
            hash: base64::encode(hash),
            salt: base64::encode(salt),
            iterations,
        })
    }

    pub fn verify(&self, password: &str) -> bool {
        let salt_result = base64::decode(&self.salt);
        let hash_result = base64::decode(&self.hash);

        match (salt_result, hash_result) {
            (Ok(salt), Ok(hash)) => pbkdf2::verify(
                PBKDF2_HMAC_SHA512,
                self.iterations,
                salt.as_slice(),
                password.as_bytes(),
                hash.as_slice(),
            )
            .is_ok(),
            _ => false,
        }
    }

    fn salt(size: usize) -> Result<Vec<u8>, ring::error::Unspecified> {
        let mut salt = vec![0; size];
        let rng = SystemRandom::new();
        rng.fill(&mut salt)?;

        Ok(salt)
    }
}
