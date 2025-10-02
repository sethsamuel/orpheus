use serenity::all::{ActivityData, UserId};

use crate::discord::thread;
use crate::poll::Poll;
use crate::types::{Context, Error, OrpheusStatus};

#[tracing::instrument]
#[poise::command(slash_command, prefix_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Users"]
    #[rest]
    users: String,
) -> Result<(), Error> {
    let mut status = ctx.data().status.lock().await;
    *status = OrpheusStatus::Processing;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom("Processing...")));

    let (poll_option, thread_message) = thread::get::<Poll>(ctx).await;
    let mut poll = poll_option.unwrap();
    if poll.host != ctx.author().id {
        _ = ctx
            .reply(format!(
                "Sorry, only the host (<@{}>) can add users",
                poll.host
            ))
            .await;
        return Ok(());
    }

    let re = regex::Regex::new(r"<@(\d+)>").unwrap();
    re.captures_iter(&users)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str())
        .filter_map(|s| s.parse::<u64>().ok())
        .for_each(|u| _ = poll.required_users.insert(UserId::new(u)));

    _ = poll
        .update_message(ctx.http(), thread_message.channel_id, thread_message.id)
        .await;

    *status = OrpheusStatus::Waiting;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(status.as_str())));

    Ok(())
}
