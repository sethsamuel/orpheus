use crate::types::{Context, Error};
use chrono::{Days, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike};

#[derive(serde::Serialize)]
struct ThreadOptions {
    name: String,
}

enum NumberEmojis {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl NumberEmojis {
    fn as_str(&self) -> &'static str {
        match self {
            NumberEmojis::One => "1️⃣",
            NumberEmojis::Two => "2️⃣",
            NumberEmojis::Three => "3️⃣",
            NumberEmojis::Four => "4️⃣",
            NumberEmojis::Five => "5️⃣",
            NumberEmojis::Six => "6️⃣",
            NumberEmojis::Seven => "7️⃣",
        }
    }
}

const NUMBERS: &[NumberEmojis] = &[
    NumberEmojis::One,
    NumberEmojis::Two,
    NumberEmojis::Three,
    NumberEmojis::Four,
    NumberEmojis::Five,
    NumberEmojis::Six,
    NumberEmojis::Seven,
];

#[poise::command(slash_command)]
pub async fn start_thread(
    ctx: Context<'_>,
    #[description = "Event name"] name: String,
    #[description = "Start date (mm/dd/yy)"] start_date: Option<String>,
    #[description = "Poll hours"] hours: Option<isize>,
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
            let start = start_date
                .with_time(NaiveTime::from_hms_opt(19, 0, 0).unwrap())
                .unwrap();

            let format = "%a, %b %d at %I%P";
            let message =
                c.id.say(
                    ctx.http(),
                    format!(
                        "Welcome to the scheduling thread for {}!

Event voting will be open for {} hours.

{one} {day_one}
{two} {day_two}
{three} {day_three}
{four} {day_four}
{five} {day_five}
{six} {day_six}
{seven} {day_seven}
",
                        name,
                        hours.unwrap_or(24),
                        one = NumberEmojis::One.as_str(),
                        day_one = start.checked_add_days(Days::new(1)).unwrap().format(format),
                        two = NumberEmojis::Two.as_str(),
                        day_two = start.checked_add_days(Days::new(2)).unwrap().format(format),
                        three = NumberEmojis::Three.as_str(),
                        day_three = start.checked_add_days(Days::new(3)).unwrap().format(format),
                        four = NumberEmojis::Four.as_str(),
                        day_four = start.checked_add_days(Days::new(4)).unwrap().format(format),
                        five = NumberEmojis::Five.as_str(),
                        day_five = start.checked_add_days(Days::new(5)).unwrap().format(format),
                        six = NumberEmojis::Six.as_str(),
                        day_six = start.checked_add_days(Days::new(6)).unwrap().format(format),
                        seven = NumberEmojis::Seven.as_str(),
                        day_seven = start.checked_add_days(Days::new(7)).unwrap().format(format),
                    ),
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
