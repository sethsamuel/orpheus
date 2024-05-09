use serenity::all::{ActivityData, AutoArchiveDuration, EditThread, GetMessages};

use crate::poll::Poll;
use crate::types::{Context, Error, OrpheusStatus};

#[tracing::instrument]
#[poise::command(prefix_command)]
pub async fn close(ctx: Context<'_>) -> Result<(), Error> {
    let mut status = ctx.data().status.lock().await;
    *status = OrpheusStatus::Processing;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom("Processing...")));

    let thread = ctx
        .guild_channel()
        .await
        .unwrap()
        .messages(ctx.http(), GetMessages::new())
        .await
        .unwrap();
    let thread_message = thread.last().unwrap().clone();
    let poll = Poll::try_from(thread_message.content.clone()).unwrap();
    println!("{:?}", poll);
    if poll.host != ctx.author().id {
        let _ = ctx
            .reply(format!(
                "Sorry, only the host (<@{}>) can close the thread",
                poll.host
            ))
            .await;
        return Ok(());
    }

    let _ = ctx
        .reply("This thread is locked and will be archived in one day.")
        .await;

    let _ = ctx
        .http()
        .edit_thread(
            ctx.channel_id(),
            &EditThread::new()
                .locked(true)
                .auto_archive_duration(AutoArchiveDuration::OneDay),
            Some("Voting closed"),
        )
        .await
        .inspect_err(|e| println!("Error closing thread {}", e))
        .inspect(|_| println!("Channel archived"));

    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom("Waiting...")));

    Ok(())
}
