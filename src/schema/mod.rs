use async_graphql::*;

use self::{mutation::auth::AuthMutation, query::{auth::AuthQuery, game::GameQuery, user::UserQuery}};

pub mod types;
mod query;
mod mutation;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(AuthQuery, GameQuery, UserQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(AuthMutation);
