use dotenvy::dotenv;

use tokio::sync::Mutex;

use poise::serenity_prelude as serenity;

mod types;
use types::{Error, State};

mod commands;
mod handlers;
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
                    lock: Mutex::new(()),
                    status: Mutex::new(types::OrpheusStatus::Waiting),
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
        serenity::FullEvent::Ready { data_about_bot, .. } => handlers::on_ready(data_about_bot),
        serenity::FullEvent::Message { new_message } => {
            handlers::on_message(new_message, ctx, event, data).await
        }
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            handlers::on_reaction_change(add_reaction, ctx, event, data).await
        }
        serenity::FullEvent::ReactionRemove { removed_reaction } => {
            handlers::on_reaction_change(removed_reaction, ctx, event, data).await
        }

        _ => Ok(()),
    }
}
