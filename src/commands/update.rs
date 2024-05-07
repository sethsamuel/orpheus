use serenity::all::GetMessages;

use crate::poll::consts::NUMBERS;
use crate::poll::Poll;
use crate::types::{Context, Error, OrpheusStatus};

#[poise::command(prefix_command)]
pub async fn update(ctx: Context<'_>) -> Result<(), Error> {
    let mut status = ctx.data().status.lock().await;
    *status = OrpheusStatus::Processing;
    let bot_id = ctx.http().get_current_user().await.unwrap().id;

    let channel = ctx.guild_channel().await.unwrap();

    if channel.thread_metadata.unwrap().locked {
        return Ok(());
    }

    let thread = channel
        .messages(ctx.http(), GetMessages::new())
        .await
        .unwrap();
    let message = thread.last().unwrap().clone();
    let mut poll = Poll::try_from(message.content.clone()).unwrap();
    println!("{:?}", poll);

    let _ = poll
        .update_days(ctx.http(), bot_id, message.channel_id, message.id)
        .await;
    if poll.eliminated_days.len() == NUMBERS.len() {
        println!("All days eliminated!");
        poll.next_dates(ctx.http(), &message).await;
    }

    let _ = poll
        .update_message(ctx.http(), ctx.channel_id(), message.id)
        .await;

    let _ = ctx.reply("Updated!").await;

    Ok(())
}
