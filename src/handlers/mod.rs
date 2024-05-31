use ::serenity::all::ActivityData;

use ::serenity::all::CacheHttp;
use ::serenity::all::Context;

use ::serenity::all::Reaction;

use poise::serenity_prelude as serenity;

use crate::poll::consts::FINISHED;

use crate::poll::Poll;
use crate::telephone::Telephone;
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
    if *status == OrpheusStatus::Stopped {
        return Ok(());
    }

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
        *status = OrpheusStatus::Waiting;
        ctx.set_activity(Some(ActivityData::custom(status.as_str())));

        return Ok(());
    }

    let poll = Poll::try_from(message.content.clone());
    match poll.ok() {
        Some(poll) => poll.on_reaction(ctx, bot_id, message).await,
        None => {
            let telephone = Telephone::try_from(message.content.clone());
            match telephone.ok() {
                Some(telephone) => telephone.on_reaction(ctx, bot_id, message).await,
                None => println!("Reaction to undecodable message {:?}", message.content),
            }
        }
    }

    *status = OrpheusStatus::Waiting;
    ctx.set_activity(Some(ActivityData::custom(status.as_str())));

    Ok(())
}
