use chrono::{Duration, TimeZone, Utc};
use ring::rand::{SecureRandom, SystemRandom};

use super::get_refresh_token_key;

pub enum RefreshCheckResult {
    OnlyAccessToken,
    Both,
    None,
}

pub fn create_refresh_token(size: usize) -> Option<String> {
    let mut buf: Vec<u8> = vec![0; size];
    let rng = SystemRandom::new();
    rng.fill(&mut buf).ok()?;

    Some(base64::encode(&buf))
}

pub async fn register_refresh_token(
    refresh_token: &str,
    redis_conn: &mut deadpool_redis::Connection,
) -> Result<(), redis::RedisError> {
    let key = get_refresh_token_key(refresh_token);
    let expire_at = (Utc::now() + Duration::days(30)).timestamp().to_string();

    redis::pipe()
        .cmd("SET")
        .arg(&[&key, &expire_at])
        .ignore()
        .cmd("EXPIREAT")
        .arg(&[&key, &expire_at])
        .query_async(redis_conn)
        .await
}

pub async fn check_refresh(
    refresh_token: &str,
    redis_conn: &mut deadpool_redis::Connection,
) -> Result<RefreshCheckResult, redis::RedisError> {
    let result = redis::cmd("GET")
        .arg(get_refresh_token_key(refresh_token))
        .query_async::<_, Option<i64>>(redis_conn)
        .await?;

    if let Some(expire_at) = result {
        let expire_at = Utc.timestamp(expire_at, 0);
        let diff = expire_at.signed_duration_since(Utc::now());

        match diff < Duration::days(10) {
            true => Ok(RefreshCheckResult::Both),
            false => Ok(RefreshCheckResult::OnlyAccessToken),
        }
    } else {
        Ok(RefreshCheckResult::None)
    }
}
