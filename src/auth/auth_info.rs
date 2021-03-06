use std::{convert::Infallible, pin::Pin, str::FromStr};

use actix_web::{dev::Payload, FromRequest, HttpRequest};
use futures::Future;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use uuid::Uuid;
use webgame_collection_api_macros::Error;

use crate::{auth::login::Claims, config::CONFIG};

use super::get_invalid_token_key;

pub struct AuthInfo {
    user_id: Option<Uuid>,
    auth_token: Option<String>,
    token_exp: Option<usize>,
    valid: Option<bool>,
}

impl AuthInfo {
    pub fn get_user_id(&self) -> Result<Uuid, AuthError> {
        if self.valid.unwrap_or(false) {
            self.user_id.ok_or(AuthError::NotAuthorized)
        } else {
            Err(AuthError::Invalidated)
        }
    }

    pub fn from_header(header: Option<String>) -> AuthInfo {
        match header {
            Some(header) if header.starts_with("Bearer ") => {
                let token = header.trim_start_matches("Bearer ");
                let decode_data = jsonwebtoken::decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(CONFIG.jwt_secret.as_slice()),
                    &Validation::new(Algorithm::HS512),
                )
                .map_err(anyhow::Error::new)
                .and_then(|data| {
                    Ok((
                        Uuid::from_str(&data.claims.sub).map_err(anyhow::Error::new)?,
                        data.claims.exp,
                    ))
                });
                let (user_id, token_exp) = match decode_data {
                    Ok((user_id, token_exp)) => (Some(user_id), Some(token_exp)),
                    Err(_) => (None, None),
                };

                AuthInfo {
                    user_id,
                    auth_token: Some(token.to_owned()),
                    token_exp,
                    valid: None,
                }
            }
            _ => AuthInfo {
                user_id: None,
                auth_token: None,
                token_exp: None,
                valid: None,
            },
        }
    }

    pub async fn verify(&mut self, redis_conn: &mut deadpool_redis::Connection) -> bool {
        if let Some(valid) = self.valid {
            return valid;
        }

        let valid = match &self.auth_token {
            Some(auth_token) => {
                let result = redis::cmd("GET")
                    .arg(get_invalid_token_key(auth_token))
                    .query_async::<_, Option<i64>>(redis_conn)
                    .await;

                result.map(|opt| opt.is_none()).unwrap_or(false)
            }
            None => false,
        };

        self.valid = Some(valid);
        valid
    }

    pub async fn invalidate(
        &self, // &mut self
        redis_conn: &mut deadpool_redis::Connection,
    ) -> Result<bool, redis::RedisError> {
        if let (Some(auth_token), Some(token_exp)) = (&self.auth_token, &self.token_exp) {
            let key = get_invalid_token_key(auth_token);
            let token_exp = token_exp.to_string();

            return redis::pipe()
                .cmd("SET")
                .arg(&[&key, &token_exp])
                .ignore()
                .cmd("EXPIREAT")
                .arg(&[&key, &token_exp])
                .query_async(redis_conn)
                .await
                .map(|_: ()| {
                    // ????????? AuthInfo??? HTTP Request?????? ???????????? ???????????????,
                    // GraphQL Field Resolver?????? Mutex ?????? ???????????? ?????????
                    // &mut self??? ?????? ??? ????????? ????????? ????????????
                    // ??????????????? valid ????????? ??????????????? ????????? ??????.
                    // self.valid = None;
                    true
                });
        }

        Ok(false)
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

#[derive(Error)]
pub enum AuthError {
    #[error(message = "Not authorized")]
    NotAuthorized,
    #[error(message = "Invalidated auth token")]
    Invalidated,
}
