use poise::CreateReply;
use serenity::all::GetMessages;

use crate::discord::thread;
use crate::poll::consts::NUMBERS;
use crate::poll::{Poll, ReplyContext};
use crate::telephone::Telephone;
use crate::types::{Context, Error, OrpheusStatus};

#[tracing::instrument]
#[poise::command(slash_command, prefix_command)]
pub async fn update(ctx: Context<'_>) -> Result<(), Error> {
    let mut status = ctx.data().status.lock().await;
    *status = OrpheusStatus::Processing;
    let bot_id = ctx.http().get_current_user().await.unwrap().id;

    let reply = ctx.reply("Updating...").await.unwrap();

    let channel = ctx.guild_channel().await.unwrap();

    if channel.thread_metadata.unwrap().locked {
        *status = OrpheusStatus::Waiting;
        let _ = reply
            .edit(
                ctx,
                CreateReply::default()
                    .content("Thread currently being updated, please try again in a minute"),
            )
            .await;
        return Ok(());
    }

    let thread = channel
        .messages(ctx.http(), GetMessages::new())
        .await
        .unwrap();
    let message = thread.last().unwrap().clone();
    if let Ok(mut poll) = Poll::try_from(message.content.clone()) {
        _ = poll
            .update_days(
                ctx.http(),
                bot_id,
                message.channel_id,
                message.id,
                Some(ReplyContext {
                    handle: &reply,
                    ctx: &ctx,
                }),
            )
            .await;
        if poll.eliminated_days.len() == NUMBERS.len() {
            println!("All days eliminated!");
            poll.next_dates(ctx.http(), &message).await;
        }

        let _ = reply
            .edit(ctx, CreateReply::default().content("Updating thread..."))
            .await;
        _ = poll
            .update_message(ctx.http(), ctx.channel_id(), message.id)
            .await;
    } else if let Ok(mut telephone) = Telephone::try_from(message.content.clone()) {
        telephone
            .update_players(ctx.http(), bot_id, ctx.channel_id(), message.id)
            .await;
        _ = thread::update(ctx.http(), ctx.channel_id(), message.id, telephone).await;
    }
    let _ = reply
        .edit(ctx, CreateReply::default().content("Updated"))
        .await;

    Ok(())
}
