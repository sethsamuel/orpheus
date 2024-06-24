use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    time::Duration,
};

use dotenvy::dotenv;

use nagger::Nagger;
use tokio::sync::{Mutex, RwLock};

use poise::serenity_prelude as serenity;

mod types;
use types::{DiscordMessage, Error, State};

mod commands;
mod discord;
mod handlers;
mod nagger;
mod poll;
mod telephone;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    _ = dotenv();
    let (_guard, tracer_shutdown) = datadog_tracing::init()?;

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let (tx, rx): (Sender<DiscordMessage>, Receiver<DiscordMessage>) = mpsc::channel();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::orpheus::orpheus(),
                commands::start::start(),
                commands::stop::stop(),
                commands::add::add(),
                commands::update::update(),
                commands::close::close(),
                commands::next_dates::next_dates(),
                commands::nag::nag(),
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
                    tx,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    let nagger = Arc::new(RwLock::new(Nagger::new()));
    let http = client.http.clone();
    nagger.write().await.init(http).await;
    let rx_nagger = nagger.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(m) = rx.recv() {
                if !rx_nagger.read().await.messages.contains(&m) {
                    rx_nagger.write().await.messages.insert(m);
                    rx_nagger.write().await.save();
                    rx_nagger.write().await.execute().await;
                }
            }
        }
    });

    let i_nagger = nagger.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60 * 5));

        loop {
            interval.tick().await; // This should go first.
            i_nagger.write().await.execute().await;
        }
    });

    client.start().await.unwrap();

    tracer_shutdown.shutdown();

    Ok(())
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
