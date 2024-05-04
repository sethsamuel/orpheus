use std::sync::atomic::Ordering;

use ::serenity::all::CacheHttp;
use ::serenity::all::Reaction;
use poise::serenity_prelude as serenity;

use crate::poll::Poll;

use super::types::{Error, State};
use serenity::all::Message;
use serenity::all::Ready;

pub fn on_ready(data_about_bot: &Ready) -> Result<(), Error> {
    println!("Logged in as {}", data_about_bot.user.name);
    Ok(())
}

pub async fn on_message(
    message: &Message,
    ctx: &serenity::Context,
    _: &serenity::FullEvent,
    data: &State,
) -> Result<(), Error> {
    println!("New message: {}", message.content);
    if message.content.to_lowercase().contains("poise")
        && message.author.id != ctx.cache.current_user().id
    {
        let old_mentions = data.poise_mentions.fetch_add(1, Ordering::SeqCst);
        message
            .reply(
                ctx,
                format!("Poise has been mentioned {} times", old_mentions + 1),
            )
            .await?;
    }

    Ok(())
}

pub async fn on_reaction_add(
    add_reaction: &Reaction,
    ctx: &serenity::Context,
    _: &serenity::FullEvent,
    _: &State,
) -> Result<(), Error> {
    let message = add_reaction.message(ctx.http()).await.unwrap();
    let poll = Poll::try_from(message.content).unwrap();
    println!("{:?}", poll);
    println!(
        "New reaction {} to {}",
        add_reaction.emoji, add_reaction.message_id
    );

    Ok(())
}
