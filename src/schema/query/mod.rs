use async_graphql::*;

mod game;
mod node;
mod user;

#[derive(MergedObject, Default)]
pub struct QueryRoot(node::NodeQuery, game::GameQuery, user::UserQuery);
