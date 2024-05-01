use dotenvy::dotenv;
use std::sync::atomic::{AtomicU32, Ordering};

use poise::serenity_prelude as serenity;

struct Data {
    poise_mentions: AtomicU32,
} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(serde::Serialize)]
struct ThreadOptions {
    name: String,
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[poise::command(slash_command)]
async fn start_thread(ctx: Context<'_>) -> Result<(), Error> {
    println!("Starting thread");
    let res = ctx
        .http()
        .create_thread(
            ctx.channel_id(),
            &ThreadOptions {
                name: "New Thread".to_string(),
            },
            Some("Time to schedule"),
        )
        .await;
    match res {
        Ok(c) => {
            println!("Created {c}");
            let _ = c.id.say(ctx.http(), "Hi thread").await;
            let _ = ctx.say(format!("Created a new thread <#{}>", c.id)).await;
        }
        Err(err) => {
            println!("Error {err}");
            let _ = ctx.say("Something went wrong :(").await;
        }
    }
    println!("Command handled");
    Ok(())
}

#[tokio::main]
async fn main() {
    let _ = dotenv();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), start_thread()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
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
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
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
        _ => {}
    }
    Ok(())
}
