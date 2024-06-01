use serenity::all::GetMessages;

use crate::poll::Poll;
use crate::types::{Context, Error};

#[tracing::instrument]
#[poise::command(prefix_command)]
pub async fn nag(ctx: Context<'_>) -> Result<(), Error> {
    let thread = ctx
        .guild_channel()
        .await
        .unwrap()
        .messages(ctx.http(), GetMessages::new())
        .await
        .unwrap();
    let thread_message = thread.last().unwrap().clone();
    let poll = Poll::try_from(thread_message.content.clone()).unwrap();

    let finished_users = poll
        .get_finished_users(
            ctx.http(),
            thread_message.channel_id,
            thread_message.id,
            ctx.http().get_current_user().await.unwrap().id,
        )
        .await;

    let nag_users = poll
        .required_users
        .clone()
        .iter()
        .filter(|u| !finished_users.contains(u))
        .map(|u| format!("<@{}>", u))
        .collect::<Vec<_>>();

    if !nag_users.is_empty() {
        let nag_str: String = nag_users.join(" ");
        let message = format!("Sorry to bother you {}, but could you please take the time to fill out the current poll?\n\nThanks in advance, your friendly nagging bot.", nag_str);

        _ = ctx.reply(message).await;
    } else {
        _ = ctx.reply("No one left to nag, thanks everyone!").await;
    }
    Ok(())
}
