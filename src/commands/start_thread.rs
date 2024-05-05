
use crate::poll::Poll;
use crate::types::{Context, Error};
use chrono::{Days, NaiveDate, Utc};


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
    let _ = ctx.defer().await;

    let parsed = match &start_date {
        Some(str) => NaiveDate::parse_from_str(str.as_str(), "%D").ok(),
        _ => None,
    };
    let start_date = match parsed {
        Some(_) => parsed.unwrap(),
        _ => Utc::now().naive_local().date(),
    };
    let end_date = start_date
        .checked_add_days(Days::new(days.unwrap_or(1)))
        .unwrap();
    // let start = NaiveDateTime::new(start_date, NaiveTime::from_hms_opt(19, 0, 0).unwrap());

    let host: serenity::model::prelude::UserId = ctx.author().id;
    let poll = Poll {
        host,
        event_name: name,
        end_date,
        start_date,
        ..Default::default()
    };

    let (channel_id, _) = poll
        .start_thread(ctx.http(), ctx.channel_id())
        .await
        .unwrap();

    let _ = ctx
        .say(format!("Created a new thread <#{}>", channel_id))
        .await;
    println!("Starting thread");
    println!("Command handled");
    Ok(())
}
