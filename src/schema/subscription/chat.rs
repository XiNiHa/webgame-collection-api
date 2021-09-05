use async_graphql::*;
use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    auth::auth_info::AuthInfo, chat::CHANNEL_MAP, error::Error, schema::types::chat::Chat,
};

#[derive(Default)]
pub struct ChatSubscription;

#[Subscription]
impl ChatSubscription {
    async fn chats(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = Chat>> {
        let auth_info = ctx.data::<AuthInfo>()?;
        let user_id = auth_info.get_user_id().map_err(|e| e.build())?;
        let (tx, rx) = mpsc::channel::<Chat>(10);

        let mut map_guard = CHANNEL_MAP.lock().await;
        map_guard.insert(user_id, tx);

        Ok(ReceiverStream::new(rx))
    }
}
