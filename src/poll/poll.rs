use base64::Engine;
use chrono::{DateTime, Days, Local, NaiveDate, NaiveTime};

use super::{
    consts::{FINISHED, NUMBERS},
    strings::strip_zero_padding,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Poll {
    pub event_name: String,
    pub host: serenity::model::prelude::UserId,
    pub end_date: NaiveDate,
    pub start_date: NaiveDate,
}

#[derive(Debug)]
pub struct FromStringError;

impl TryFrom<String> for Poll {
    type Error = FromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut lines = value.lines();

        // let mut poll = Poll::default();
        // poll.event_name = Poll::event_name_from_line(lines.next().unwrap());
        // poll.host = Poll::host_from_line(lines.next().unwrap()).parse().unwrap();
        // poll.end_date = Poll::end_date_from_line(lines.next().unwrap());

        let line = lines.last().unwrap().trim().replace("|", "");
        let base = base64::prelude::BASE64_STANDARD.decode(line).unwrap();
        let str = String::from_utf8(base).unwrap();
        let poll = serde_json::from_str(&str).unwrap();

        Ok(poll)
    }
}

impl From<Poll> for String {
    fn from(value: Poll) -> Self {
        let mut message_str = "".to_string();
        message_str = message_str + value.welcome_line().as_str();
        message_str = message_str + "\n";
        message_str = message_str + value.host_line().as_str();
        message_str = message_str + "\n";
        message_str = message_str + value.ends_at_line().as_str();
        message_str = message_str + "\n";

        let date_format = "%a, %b %d at %I%P";

        for i in 0..NUMBERS.len() {
            let emoji = NUMBERS[i].as_str();
            let date = value
                .start_date
                .checked_add_days(Days::new(i.try_into().unwrap()))
                .unwrap()
                .and_time(NaiveTime::from_hms_opt(19, 0, 0).unwrap())
                .format(date_format)
                .to_string();
            let clean = strip_zero_padding(&date);
            message_str = message_str + format!("{emoji} {clean}\n").as_str();
        }
        message_str =
            message_str + format!("\nTo lock in your availability, hit {FINISHED}").as_str();
        message_str = message_str + "\n";
        message_str = message_str + "Orpehus Magic String (feel free to ignore):";
        message_str = message_str + "\n";
        message_str = message_str
            + "||"
            + base64::prelude::BASE64_STANDARD
                .encode(serde_json::to_string(&value).unwrap().as_bytes())
                .as_str()
            + "||";

        message_str
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDate};

    use crate::poll::poll::FromStringError;

    #[test]
    fn test_froms() {
        use crate::Poll;
        let poll = Poll {
            event_name: "My event!".to_string(),
            host: 123451234.into(),
            end_date: NaiveDate::from_ymd_opt(2024, 02, 11).unwrap(),
            start_date: NaiveDate::from_ymd_opt(2024, 02, 18).unwrap(),
        };
        let str: String = String::from(poll.clone());
        let poll2: Result<Poll, FromStringError> = str.try_into();
        assert_eq!(poll, poll2.unwrap());
    }
}
