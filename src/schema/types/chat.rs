use async_graphql::*;

#[derive(SimpleObject)]
pub struct Chat {
    pub sender_id: ID,
    pub message: String,
}
