use std::collections::HashSet;

use crate::commands::add::add;
use crate::commands::allow_truancy::allow_truancy;
use crate::commands::close::close;
use crate::commands::help::help;
use crate::commands::nag::nag;
use crate::commands::next_dates::next_dates;
use crate::commands::set_lead::set_lead;
use crate::commands::story_time::story_time;
use crate::commands::update::update;
use crate::poll::Poll;
use crate::types::{Context, Error, OrpheusStatus};
use chrono::{Days, NaiveDate, Utc};
use serenity::all::ActivityData;

#[poise::command(
    slash_command,
    subcommands(
        "help",
        "save_me",
        "add",
        "allow_truancy",
        "update",
        "nag",
        "close",
        "next_dates",
        "story_time",
        "set_lead"
    ),
    subcommand_required
)]
pub async fn orpheus(_: Context<'_>) -> Result<(), Error> {
    // This will never be called, because `subcommand_required` parameter is set
    Ok(())
}

#[poise::command(slash_command)]
pub async fn save_me(
    ctx: Context<'_>,
    #[description = "Event name"] name: String,
    #[description = "First day to poll (mm/dd/yy)"] first_poll_date: Option<String>,
) -> Result<(), Error> {
    _ = ctx.defer().await;

    let mut status = ctx.data().status.lock().await;
    if *status == OrpheusStatus::Stopped {
        return Ok(());
    }
    *status = OrpheusStatus::Processing;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom("Processing...")));

    println!("Received first_poll_date {:?}", first_poll_date);
    let parsed = match &first_poll_date {
        Some(str) => NaiveDate::parse_from_str(str.trim(), "%m/%d/%Y").ok(),
        _ => None,
    };
    let start_date = match parsed {
        Some(_) => parsed.unwrap(),
        _ => Utc::now().naive_local().date(),
    };
    println!("Received start_date {}", start_date);
    let end_date = start_date.checked_sub_days(Days::new(1)).unwrap();
    // let start = NaiveDateTime::new(start_date, NaiveTime::from_hms_opt(19, 0, 0).unwrap());

    let host: serenity::model::prelude::UserId = ctx.author().id;
    let poll = Poll {
        host,
        event_name: name,
        required_users: HashSet::new(),
        end_date,
        start_date,
        ..Default::default()
    };

    let (channel_id, _) = poll
        .start_thread(ctx.http(), ctx.channel_id())
        .await
        .unwrap();

    _ = ctx
        .say(format!("Created a new thread <#{}>", channel_id))
        .await;
    println!("Starting thread");

    *status = OrpheusStatus::Waiting;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom(status.as_str())));

    println!("Command handled");

    Ok(())
}
