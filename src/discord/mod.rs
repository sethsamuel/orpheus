use poise::serenity_prelude as serenity;
use serenity::all::{ChannelId, Http, MessageId, ReactionType};

pub mod thread;

#[tracing::instrument]
pub async fn get_reaction_users(
    http: &Http,
    channel_id: ChannelId,
    message_id: MessageId,
    r: String,
) -> Result<(ReactionType, Vec<serenity::User>), ::serenity::Error> {
    let reaction = ReactionType::Unicode(r);
    let users = http
        .get_reaction_users(channel_id, message_id, &reaction, 50, None)
        .await?;
    Ok((reaction, users))
}
