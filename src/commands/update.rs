use serenity::all::{GetMessages};

use crate::poll::Poll;
use crate::types::{Context, Error};

#[poise::command(prefix_command)]
pub async fn update(ctx: Context<'_>) -> Result<(), Error> {
    // println!(ctx);

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
    let _ = poll
        .update_message(&ctx, ctx.channel_id(), thread_message.id)
        .await;

    let _ = ctx.reply("Updated!").await;
    Ok(())
}
