use poise::CreateReply;
use serenity::all::{ActivityData, AutoArchiveDuration, EditThread};

use crate::discord::thread;
use crate::poll::Poll;
use crate::types::{Context, Error, OrpheusStatus};

#[tracing::instrument]
#[poise::command(slash_command, prefix_command)]
pub async fn archive(ctx: Context<'_>) -> Result<(), Error> {
    let mut status = ctx.data().status.lock().await;
    *status = OrpheusStatus::Processing;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom("Processing...")));

    let reply = ctx.reply("Archiving thread...").await.unwrap();

    let (poll_option, _thread_message) = thread::get::<Poll>(ctx).await;

    let Some(poll) = poll_option else {
        return Err("couldn't get thread".into());
    };
    if poll.host != ctx.author().id {
        _ = ctx
            .reply(format!(
                "Sorry, only the host (<@{}>) can archive the thread",
                poll.host
            ))
            .await;
        return Ok(());
    }

    _ = ctx
        .http()
        .edit_thread(
            ctx.channel_id(),
            &EditThread::new()
                .locked(true)
                .auto_archive_duration(AutoArchiveDuration::OneHour),
            Some("Voting closed"),
        )
        .await?;

    _ = reply
        .edit(
            ctx,
            CreateReply::default().content("Thread will be archived in one hour!"),
        )
        .await;
    *status = OrpheusStatus::Waiting;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(status.as_str())));

    Ok(())
}
