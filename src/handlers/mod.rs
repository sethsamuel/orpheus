use ::serenity::all::ActivityData;

use ::serenity::all::AutoArchiveDuration;
use ::serenity::all::CacheHttp;
use ::serenity::all::Context;

use ::serenity::all::CreateMessage;
use ::serenity::all::EditThread;
use ::serenity::all::Reaction;

use ::serenity::all::ReactionType;
use ::serenity::all::User;
use poise::serenity_prelude as serenity;

use crate::discord::thread;
use crate::poll::consts::ARCHIVE;
use crate::poll::consts::FINISHED;
use crate::poll::Poll;
use crate::telephone::Telephone;
use crate::types::DiscordMessage;
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

    if reaction.emoji.unicode_eq(ARCHIVE) {
        let message = reaction.message(ctx.http()).await.unwrap();

        // Get the poll to check the host
        if let Ok(poll) = Poll::try_from(message.content.clone()) {
            println!("{:?}", poll);
            if Some(poll.host) == reaction.user_id
                // The reaction change event triggers for both add and remove
                // so check if the host is still in the list
                && reaction
                    .users(ctx.http(), ReactionType::Unicode(ARCHIVE.into()), None, None::<User>)
                    .await?
                    .iter().any(|u|u.id == poll.host)
            {
                _ = ctx
                    .http()
                    .edit_thread(
                        message.channel_id,
                        &EditThread::new()
                            .locked(true)
                            .auto_archive_duration(AutoArchiveDuration::OneHour),
                        Some("Voting closed"),
                    )
                    .await?;
                _ = ctx
                    .http()
                    .send_message(
                        message.channel_id,
                        vec![],
                        &CreateMessage::new().content("Archiving thread in one hour!"),
                    )
                    .await
                    .unwrap();
            }
        }
    }

    // Ignore other reactions that aren't finished
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
    let message_id = message.id;
    let channel_id = message.channel_id;

    if thread::is_locked(ctx.http(), message.channel_id).await {
        *status = OrpheusStatus::Waiting;
        ctx.set_activity(Some(ActivityData::custom(status.as_str())));

        return Ok(());
    }

    if let Ok(poll) = Poll::try_from(message.content.clone()) {
        println!("{:?}", poll);
        poll.on_reaction(ctx, bot_id, reaction, message).await;
        _ = data.tx.send(DiscordMessage {
            channel_id,
            message_id,
        });
    } else if let Ok(telephone) = Telephone::try_from(message.content.clone()) {
        println!("{:?}", telephone);
        telephone.on_reaction(ctx, bot_id, reaction, message).await;
        _ = data.tx.send(DiscordMessage {
            channel_id,
            message_id,
        });
    } else {
        println!("Reaction to undecodable message {:?}", message.content)
    }

    *status = OrpheusStatus::Waiting;
    ctx.set_activity(Some(ActivityData::custom(status.as_str())));

    Ok(())
}
