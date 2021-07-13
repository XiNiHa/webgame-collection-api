use async_graphql::*;
use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    auth::auth_info::AuthInfo, chat::CHANNEL_MAP, error::Error, schema::types::chat::Chat,
};

#[derive(Debug)]
enum ChatSubscriptionError {
    NotAuthorized,
}

impl Error for ChatSubscriptionError {
    fn message(&self) -> String {
        match self {
            ChatSubscriptionError::NotAuthorized => "Not Authorized",
        }
        .to_owned()
    }

    fn code(&self) -> String {
        format!("ChatSubscriptionError::{:?}", self)
    }
}

#[derive(Default)]
pub struct ChatSubscription;

#[Subscription]
impl ChatSubscription {
    async fn chats(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = Chat>> {
        let AuthInfo { user_id } = ctx.data::<AuthInfo>()?;
        let (tx, rx) = mpsc::channel::<Chat>(10);

        let mut map_guard = CHANNEL_MAP.lock().await;
        map_guard.insert(
            user_id.ok_or(ChatSubscriptionError::NotAuthorized.build())?,
            tx,
        );

        Ok(ReceiverStream::new(rx))
    }
}
