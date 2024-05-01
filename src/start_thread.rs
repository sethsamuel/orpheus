use crate::types::{Context, Error};

#[derive(serde::Serialize)]
struct ThreadOptions {
    name: String,
}

enum NumberEmojis {
    One,
    Two,
}

impl NumberEmojis {
    fn as_str(&self) -> &'static str {
        match self {
            NumberEmojis::One => "1️⃣",
            NumberEmojis::Two => "2️⃣",
        }
    }
}

const NUMBERS: &[NumberEmojis] = &[NumberEmojis::One, NumberEmojis::Two];

#[poise::command(slash_command)]
pub async fn start_thread(
    ctx: Context<'_>,
    #[description = "Event name"] name: String,
    #[description = "Poll hours"] hours: Option<isize>,
) -> Result<(), Error> {
    println!("Starting thread");
    let res = ctx
        .http()
        .create_thread(
            ctx.channel_id(),
            &ThreadOptions {
                name: name.to_string(),
            },
            Some("Time to schedule"),
        )
        .await;
    match res {
        Ok(c) => {
            println!("Created {c}");
            let message =
                c.id.say(
                    ctx.http(),
                    format!("Welcome to the scheduling thread for {}!\n\nEvent voting will be open for {} hours.\n\n{one} Wed May 1, 7PM EST", name, hours.unwrap_or(24), one = NumberEmojis::One.as_str()),
                )
                .await;
            let message_id = message.unwrap().id;
            for n in NUMBERS {
                let _ = ctx
                    .http()
                    .create_reaction(
                        c.id,
                        message_id,
                        &serenity::all::ReactionType::Unicode(n.as_str().to_string()),
                    )
                    .await;
            }
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
