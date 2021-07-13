use async_graphql::*;

use super::scalars::UuidScalar;

#[derive(SimpleObject)]
pub struct Chat {
    pub sender_id: UuidScalar,
    pub message: String,
}
