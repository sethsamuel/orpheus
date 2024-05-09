use crate::poll::Poll;
use crate::types::{Context, Error, OrpheusStatus};
use chrono::{NaiveDate, Utc};
use serenity::all::{ActivityData, User, UserId};

#[poise::command(slash_command, subcommands("save_me"), subcommand_required)]
pub async fn orpheus(_: Context<'_>) -> Result<(), Error> {
    // This will never be called, because `subcommand_required` parameter is set
    Ok(())
}

#[poise::command(slash_command)]
pub async fn save_me(
    ctx: Context<'_>,
    #[description = "Event name"] name: String,
    #[description = "Required users"] required_users: Vec<User>,
    #[description = "First day to poll (mm/dd/yy)"] first_poll_date: Option<String>,
) -> Result<(), Error> {
    let _ = ctx.defer().await;

    let mut status = ctx.data().status.lock().await;
    if *status == OrpheusStatus::Stopped {
        return Ok(());
    }
    *status = OrpheusStatus::Processing;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom("Processing...")));

    let parsed = match &first_poll_date {
        Some(str) => NaiveDate::parse_from_str(str.as_str(), "%D").ok(),
        _ => None,
    };
    let start_date = match parsed {
        Some(_) => parsed.unwrap(),
        _ => Utc::now().naive_local().date(),
    };
    let end_date = start_date;
    // let start = NaiveDateTime::new(start_date, NaiveTime::from_hms_opt(19, 0, 0).unwrap());

    let host: serenity::model::prelude::UserId = ctx.author().id;
    let poll = Poll {
        host,
        event_name: name,
        required_users: Some(required_users.iter().map(|u| u.id).collect::<Vec<UserId>>()),
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

    *status = OrpheusStatus::Waiting;
    ctx.serenity_context()
        .set_activity(Some(ActivityData::custom("Waiting...")));

    println!("Command handled");

    Ok(())
}
