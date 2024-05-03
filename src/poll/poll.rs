use chrono::{DateTime, Days, Local};

use super::consts::{FINISHED, NUMBERS};

#[derive(Default, Debug)]
pub struct Poll {
    pub event_name: String,
    pub host: serenity::model::prelude::UserId,
    pub end_date: DateTime<Local>,
    pub start_date: DateTime<Local>,
}

#[derive(Debug)]
pub struct FromStringError;

impl TryFrom<String> for Poll {
    type Error = FromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lines = value.lines();
        if lines.count() < 1 {
            return Err(FromStringError {});
        }
        Ok(Poll::default())
    }
}

impl From<Poll> for String {
    fn from(value: Poll) -> Self {
        let name = value.event_name;
        let host = value.host;
        let end_date = value.end_date.format("%a, %b %d at %I%P");

        let mut message_str = format!(
            "Welcome to the scheduling thread for {name}!
Host: <@{host}>
Event voting will be open until {end_date}.
            
"
        );
        let date_format = "%a, %b %d at %I%P";

        for i in 0..NUMBERS.len() {
            let emoji = NUMBERS[i].as_str();
            let date = value
                .start_date
                .checked_add_days(Days::new(i.try_into().unwrap()))
                .unwrap()
                .format(date_format)
                .to_string();
            message_str = message_str + format!("{emoji} {date}\n").as_str();
        }
        message_str =
            message_str + format!("\nTo lock in your availability, hit {FINISHED}").as_str();

        message_str
    }
}
