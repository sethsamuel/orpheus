


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
    _message: &Message,
    _ctx: &serenity::Context,
    _event: &serenity::FullEvent,
    _data: &State,
) -> Result<(), Error> {
    Ok(())
}

pub async fn on_reaction_change(
    reaction: &Reaction,
    ctx: &serenity::Context,
    _: &serenity::FullEvent,
    data: &State,
) -> Result<(), Error> {
    let bot_id = ctx.http().get_current_user().await.unwrap().id;
    if reaction.user_id.unwrap() == bot_id {
        return Ok(());
    }

    let _lock = data.lock.lock();
    let message = reaction.message(ctx.http()).await.unwrap();
    let poll = Poll::try_from(message.content).unwrap();
    println!("{:?}", poll);
    println!(
        "New reaction by {} {} to {}",
        reaction.user_id.unwrap(),
        reaction.emoji,
        reaction.message_id
    );

    Ok(())
}
