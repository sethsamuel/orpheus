use poise::CreateReply;
use serenity::all::{ActivityData, UserId};

use crate::discord::thread;
use crate::poll::Poll;
use crate::types::{Context, Error, OrpheusStatus};

#[tracing::instrument]
#[poise::command(prefix_command)]
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

    let reply = ctx.say("Getting current poll status...").await.unwrap();

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

    reply
        .edit(ctx, CreateReply::default().content("Adding users..."))
        .await
        .unwrap();

    let re = regex::Regex::new(r"<@(\d+)>").unwrap();
    re.captures_iter(&users)
        .filter_map(|c| c.get(1))
        .map(|m| m.as_str())
        .filter_map(|s| s.parse::<u64>().ok())
        .for_each(|u| _ = poll.required_users.insert(UserId::new(u)));

    reply
        .edit(ctx, CreateReply::default().content("Updating thread..."))
        .await
        .unwrap();

    _ = poll
        .update_message(ctx.http(), thread_message.channel_id, thread_message.id)
        .await;

    reply
        .edit(ctx, CreateReply::default().content("Users added!"))
        .await
        .unwrap();

    *status = OrpheusStatus::Waiting;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(status.as_str())));

    Ok(())
}
