use serenity::all::{EditMessage, GetMessages};

use crate::poll::consts::{FINISHED, NUMBERS};
use crate::poll::poll::Poll;
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
    let mut thread_message = thread.last().unwrap().clone();
    let poll = Poll::try_from(thread_message.content.clone()).unwrap();
    println!("{:?}", poll);
    let new_content: String = poll.try_into().unwrap();
    let _ = thread_message
        .edit(ctx.http(), EditMessage::new().content(new_content))
        .await;

    let _ = ctx.reply("Updated!").await;
    Ok(())
}
