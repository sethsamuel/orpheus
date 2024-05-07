use ::serenity::all::ActivityData;

use ::serenity::all::CacheHttp;
use ::serenity::all::Context;

use ::serenity::all::Reaction;

use poise::serenity_prelude as serenity;

use crate::poll::consts::FINISHED;

use crate::poll::consts::NUMBERS;
use crate::poll::Poll;
use crate::types::OrpheusStatus;

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

#[tracing::instrument]
pub async fn on_reaction_change(
    reaction: &Reaction,
    ctx: &Context,
    _: &serenity::FullEvent,
    data: &State,
) -> Result<(), Error> {
    let bot_id = ctx.http().get_current_user().await.unwrap().id;
    if reaction.user_id.unwrap() == bot_id {
        return Ok(());
    }
    if !reaction.emoji.unicode_eq(FINISHED) {
        return Ok(());
    }

    let mut status = data.status.lock().await;
    *status = OrpheusStatus::Processing;
    ctx.set_activity(Some(ActivityData::custom("Processing...")));

    let _lock = data.lock.lock().await;
    let message = reaction.message(ctx.http()).await.unwrap();

    if message
        .channel(ctx.http())
        .await
        .unwrap()
        .guild()
        .unwrap()
        .thread_metadata
        .unwrap()
        .locked
    {
        ctx.set_activity(Some(ActivityData::custom("Waiting...")));

        return Ok(());
    }

    let mut poll = Poll::try_from(message.content.clone()).unwrap();

    let _ = poll
        .update_days(ctx.http(), bot_id, message.channel_id, message.id)
        .await;
    if poll.eliminated_days.len() == NUMBERS.len() {
        println!("All days eliminated!");
        poll.next_dates(ctx.http(), &message).await;
    }

    let _ = poll
        .update_message(ctx.http(), message.channel_id, message.id)
        .await;

    ctx.set_activity(Some(ActivityData::custom("Waiting...")));

    Ok(())
}
