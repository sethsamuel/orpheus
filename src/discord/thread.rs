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

#[tracing::instrument]
pub async fn get<T>(ctx: Context<'_>) -> (Option<T>, Message)
where
    T: TryFrom<String>,
    T::Error: Debug,
{
    let thread = ctx.guild_channel().await.unwrap();
    if thread.thread_metadata.is_none() {
        println!("Tried to get thread but got other channel {:?}", thread);
        return (
            None,
            thread
                .messages(ctx.http(), GetMessages::new())
                .await
                .unwrap()
                .last()
                .unwrap()
                .clone(),
        );
    }
    let messages = thread
        .messages(ctx.http(), GetMessages::new())
        .await
        .unwrap();
    let thread_message = messages.last().unwrap().clone();
    let object = try_from(&thread_message);

    (object, thread_message)
}

#[tracing::instrument]
pub fn try_from<T>(message: &Message) -> Option<T>
where
    T: TryFrom<String>,
    T::Error: Debug,
{
    T::try_from(message.content.clone()).ok()
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
    _ = http
        .edit_message(
            channel_id,
            message_id,
            &EditMessage::new().content(new_content),
            vec![],
        )
        .await;

    Ok(())
}

pub async fn is_locked(http: &Http, channel_id: ChannelId) -> bool {
    http.get_channel(channel_id)
        .await
        .unwrap()
        .guild()
        .unwrap()
        .thread_metadata
        .unwrap()
        .locked
}
