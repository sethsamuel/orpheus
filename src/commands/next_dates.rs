use serenity::all::{ActivityData, GetMessages};

use crate::poll::Poll;
use crate::types::{Context, Error, OrpheusStatus};

#[tracing::instrument]
#[poise::command(slash_command, prefix_command)]
pub async fn next_dates(ctx: Context<'_>) -> Result<(), Error> {
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
                "Sorry, only the host (<@{}>) can trigger the next dates",
                poll.host
            ))
            .await;
        return Ok(());
    }

    poll.next_dates(ctx.http(), &thread_message).await;

    *status = OrpheusStatus::Waiting;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(status.as_str())));

    Ok(())
}
