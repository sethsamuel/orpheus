use poise::CreateReply;
use serenity::all::MessageBuilder;

use crate::discord::thread;
use crate::poll::Poll;
use crate::types::{Context, Error};

#[tracing::instrument]
#[poise::command(slash_command, prefix_command)]
pub async fn allow_truancy(
    ctx: Context<'_>,
    #[description = "Number of allowed truants"]
    #[rest]
    count: String,
) -> Result<(), Error> {
    let (poll_option, thread_message) = thread::get::<Poll>(ctx).await;
    let mut poll = poll_option.unwrap();
    if poll.host != ctx.author().id {
        _ = ctx
            .reply(format!(
                "Sorry, only the host (<@{}>) can change truancy",
                poll.host
            ))
            .await;
        return Ok(());
    }

    let reply = ctx.reply("ok, updating truancy").await;

    let c = count.parse::<usize>();
    match c {
        Ok(count) => {
            poll.allowed_truants = count;
            _ = reply
                .as_ref()
                .unwrap()
                .edit(
                    ctx,
                    CreateReply::default().content("truancy updated, updating available days..."),
                )
                .await;

            let bot_id = ctx.http().get_current_user().await.unwrap().id;

            _ = poll
                .update_days(
                    ctx.http(),
                    bot_id,
                    thread_message.channel_id,
                    thread_message.id,
                )
                .await;
            _ = reply
                .unwrap()
                .edit(
                    ctx,
                    CreateReply::default().content("days updated, updating message..."),
                )
                .await;

            _ = poll
                .update_message(ctx.http(), thread_message.channel_id, thread_message.id)
                .await;
            _ = ctx.reply("Allowed truants count updated!").await.unwrap()
        }
        Err(_) => {
            _ = ctx
                .reply(format!(
                    "Invalid count specified. {} is not a positive integer",
                    count
                ))
                .await
                .unwrap();
        }
    }

    Ok(())
}
