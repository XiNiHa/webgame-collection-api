use async_graphql::*;

pub mod auth;
pub mod chat;

#[derive(MergedObject, Default)]
pub struct MutationRoot(auth::AuthMutation, chat::ChatMutation);
