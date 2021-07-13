use async_graphql::*;

mod auth;
mod game;
mod user;

#[derive(MergedObject, Default)]
pub struct QueryRoot(auth::AuthQuery, game::GameQuery, user::UserQuery);
