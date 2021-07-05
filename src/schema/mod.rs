use std::num::NonZeroU32;

use async_graphql::*;
use sqlx::PgPool;

use self::{mutation::auth::AuthMutation, query::{auth::AuthQuery, game::GameQuery}};

pub mod types;
mod query;
mod mutation;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(MergedObject, Default)]
pub struct QueryRoot(AuthQuery, GameQuery);

#[derive(MergedObject, Default)]
pub struct MutationRoot(AuthMutation);

pub struct AppContext {
    pub pool: PgPool,
    pub config: AppConfig,
}

pub struct AppConfig {
    pub pbkdf2_salt_size: usize,
    pub pbkdf2_iterations: NonZeroU32,
}
