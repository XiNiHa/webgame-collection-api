use async_graphql::*;

mod chat;

#[derive(MergedSubscription, Default)]
pub struct SubscriptionRoot(chat::ChatSubscription);
