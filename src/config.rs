use std::{env, num::NonZeroU32};

pub struct AppConfig {
    pub database_url: String,
    pub pbkdf2_salt_size: usize,
    pub pbkdf2_iterations: NonZeroU32,
    pub jwt_secret: Vec<u8>,
    pub refresh_token_size: usize,
}

lazy_static::lazy_static! {
    pub static ref CONFIG: AppConfig = {
        dotenv::dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(var) => var,
            Err(_) => panic!("Environment variable \"DATABASE_URL\" not set"),
        };
        let pbkdf2_salt_size = match env::var("PBKDF2_SALT_SIZE") {
            Ok(var) => match var.parse::<usize>() {
                Ok(size) => size,
                Err(_) => panic!("Failed to parse environment variable \"PBKDF2_SALT_SIZE\""),
            },
            Err(_) => panic!("Environment variable \"PBKDF2_SALT_SIZE\" not set"),
        };
        let pbkdf2_iterations = match env::var("PBKDF2_ITERATIONS") {
            Ok(var) => match var.parse::<NonZeroU32>() {
                Ok(iterations) => iterations,
                Err(_) => panic!("Failed to parse environment variable \"PBKDF2_ITERATIONS\""),
            },
            Err(_) => panic!("Environment variable \"PBKDF2_ITERATIONS\" not set"),
        };
        let jwt_secret = match env::var("JWT_SECRET") {
            Ok(var) => match base64::decode(var) {
                Ok(decoded) => decoded,
                Err(_) => panic!("Failed to parse environment variable \"JWT_SECRET\" which should be a valid base64 string")
            },
            Err(_) => panic!("Environment variable \"JWT_SECRET\" not set"),
        };
        let refresh_token_size = match env::var("REFRESH_TOKEN_SIZE") {
            Ok(var) => match var.parse::<usize>() {
                Ok(size) => size,
                Err(_) => panic!("Failed to parse environment variable \"REFRESH_TOKEN_SIZE\""),
            },
            Err(_) => panic!("Environment variable \"REFRESH_TOKEN_SIZE\" not set"),
        };

        AppConfig {
            database_url,
            pbkdf2_salt_size,
            pbkdf2_iterations,
            jwt_secret,
            refresh_token_size,
        }
    };
}
