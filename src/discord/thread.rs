use crate::types::Context;
use serenity::all::{
    ChannelId, ChannelType, EditMessage, GetMessages, GuildChannel, Http, Message, MessageId,
};
use std::fmt::Debug;

pub async fn create(
    http: &Http,
    channel_id: ChannelId,
    name: &str,
    object: impl Into<String>,
) -> (GuildChannel, MessageId) {
    let c = http
        .create_thread(
            channel_id,
            &serenity::builder::CreateThread::new(name).kind(ChannelType::PublicThread),
            Some("New Orpheus Thread"),
        )
        .await
        .unwrap();

    let message = c.id.say(http, object).await;
    let message_id = message.unwrap().id;

    (c, message_id)
}

pub async fn get<T>(ctx: Context<'_>) -> (T, Message)
where
    T: TryFrom<String>,
    T::Error: Debug,
{
    let thread = ctx
        .guild_channel()
        .await
        .unwrap()
        .messages(ctx.http(), GetMessages::new())
        .await
        .unwrap();
    let thread_message = thread.last().unwrap().clone();
    let object = T::try_from(thread_message.content.clone()).unwrap();

    (object, thread_message)
}

pub struct UpdateError;
#[tracing::instrument]
pub async fn update(
    http: &Http,
    channel_id: ChannelId,
    message_id: MessageId,
    object: impl Into<String> + Debug,
) -> Result<(), UpdateError> {
    let new_content: String = object.into();
    let _ = http
        .edit_message(
            channel_id,
            message_id,
            &EditMessage::new().content(new_content),
            vec![],
        )
        .await;

    Ok(())
}
