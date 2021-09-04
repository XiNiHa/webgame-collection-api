use std::{convert::Infallible, pin::Pin, str::FromStr};

use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::Future;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use uuid::Uuid;

use crate::{auth::login::Claims, config::CONFIG};

pub struct AuthInfo {
    pub user_id: Option<Uuid>,
}

impl AuthInfo {
    pub fn from_header(header: Option<String>) -> AuthInfo {
        match header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = header.trim_start_matches("Bearer ");

                AuthInfo {
                    user_id: jsonwebtoken::decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(CONFIG.jwt_secret.as_slice()),
                        &Validation::new(Algorithm::HS512),
                    )
                    .map_err(anyhow::Error::new)
                    .and_then(|data| Uuid::from_str(&data.claims.sub).map_err(anyhow::Error::new))
                    .ok(),
                }
            }
            _ => AuthInfo { user_id: None },
        }
    }
}

impl FromRequest for AuthInfo {
    type Error = Infallible;
    type Config = ();
    type Future = Pin<Box<dyn Future<Output = Result<AuthInfo, Infallible>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok().map(|s| s.to_string()));

        Box::pin(async move { Ok(AuthInfo::from_header(auth_header)) })
    }
}
