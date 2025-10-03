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

    let c = count.parse::<usize>();
    match c {
        Ok(count) => {
            poll.allowed_truants = count;
            let bot_id = ctx.http().get_current_user().await.unwrap().id;

            _ = poll.update_days(
                ctx.http(),
                bot_id,
                thread_message.channel_id,
                thread_message.id,
            );
            _ = poll.update_message(ctx.http(), thread_message.channel_id, thread_message.id);
            _ = ctx.reply("Allowed truants count updated!")
        }
        Err(_) => {
            _ = ctx.reply(format!(
                "Invalid count specified. {} is not a positive integer",
                count
            ));
        }
    }

    Ok(())
}
