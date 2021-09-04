use async_graphql::*;

mod node;
mod auth;
mod game;
mod user;

#[derive(MergedObject, Default)]
pub struct QueryRoot(node::NodeQuery, auth::AuthQuery, game::GameQuery, user::UserQuery);
