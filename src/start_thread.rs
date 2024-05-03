use crate::poll::consts::{FINISHED, NUMBERS};
use crate::poll::poll::Poll;
use crate::types::{Context, Error};
use chrono::{Days, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

#[derive(serde::Serialize)]
struct ThreadOptions {
    name: String,
}

#[poise::command(slash_command)]
pub async fn start_thread(
    ctx: Context<'_>,
    #[description = "Event name"] name: String,
    #[description = "Start date (mm/dd/yy)"] start_date: Option<String>,
    #[description = "Poll open for days"] days: Option<u64>,
) -> Result<(), Error> {
    println!("Starting thread");
    let _ = ctx.defer().await;
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
            let parsed = match &start_date {
                Some(str) => NaiveDate::parse_from_str(str.as_str(), "%D").ok(),
                _ => None,
            };
            let start_date = match parsed {
                Some(start) => Local
                    .from_local_datetime(&NaiveDateTime::new(
                        start,
                        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                    ))
                    .unwrap(),
                _ => Local::now(),
            };
            let end_date = start_date
                .checked_add_days(Days::new(days.unwrap_or(1)))
                .unwrap();
            let start = start_date
                .with_time(NaiveTime::from_hms_opt(19, 0, 0).unwrap())
                .unwrap();

            let host: serenity::model::prelude::UserId = ctx.author().id;
            let poll = Poll {
                host: host,
                event_name: name,
                end_date: end_date,
                start_date: start,
            };
            let message_str: String = poll.into();

            let message = c.id.say(ctx.http(), message_str).await;
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
            let _ = ctx
                .http()
                .create_reaction(
                    c.id,
                    message_id,
                    &serenity::all::ReactionType::Unicode(FINISHED.to_string()),
                )
                .await;

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
