use async_graphql::*;

use self::{mutation::MutationRoot, query::QueryRoot, subscription::SubscriptionRoot};

pub mod mutation;
pub mod query;
pub mod subscription;
pub mod types;

pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;
