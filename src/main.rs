use ::serenity::all::CacheHttp;
use dotenvy::dotenv;
use std::sync::atomic::{AtomicU32, Ordering};

use poise::serenity_prelude as serenity;

mod types;
use types::{Error, State};

use crate::poll::poll::Poll;
mod commands;
mod poll;

#[tokio::main]
async fn main() {
    let _ = dotenv();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::start_thread::start_thread(),
                commands::update::update(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(State {
                    poise_mentions: AtomicU32::new(0),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, State, Error>,
    data: &State,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            println!("New message: {}", new_message.content);
            if new_message.content.to_lowercase().contains("poise")
                && new_message.author.id != ctx.cache.current_user().id
            {
                let old_mentions = data.poise_mentions.fetch_add(1, Ordering::SeqCst);
                new_message
                    .reply(
                        ctx,
                        format!("Poise has been mentioned {} times", old_mentions + 1),
                    )
                    .await?;
            }
        }
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            let message = add_reaction.message(ctx.http()).await.unwrap();
            let poll = Poll::try_from(message.content).unwrap();
            println!("{:?}", poll);
            println!(
                "New reaction {} to {}",
                add_reaction.emoji, add_reaction.message_id
            )
        }
        _ => {}
    }
    Ok(())
}
