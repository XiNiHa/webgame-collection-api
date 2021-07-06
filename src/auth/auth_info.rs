use std::str::FromStr;

use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use rocket::{
    request::{FromRequest, Outcome},
    Request,
};
use sqlx::types::Uuid;

use crate::{auth::login::Claims, config::CONFIG};

pub struct AuthInfo {
    pub user_id: Option<Uuid>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthInfo {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        fn is_bearer(value: &str) -> bool {
            value.starts_with("Bearer ")
        }

        match request.headers().get_one("Authorization") {
            Some(value) if is_bearer(value) => {
                let token = value.trim_start_matches("Bearer ");

                Outcome::Success(AuthInfo {
                    user_id: jsonwebtoken::decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(CONFIG.jwt_secret.as_slice()),
                        &Validation::new(Algorithm::HS512),
                    )
                    .map_err(anyhow::Error::new)
                    .and_then(|data| Uuid::from_str(&data.claims.sub).map_err(anyhow::Error::new))
                    .ok(),
                })
            }
            _ => Outcome::Success(AuthInfo { user_id: None }),
        }
    }
}
