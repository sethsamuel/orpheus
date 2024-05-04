use chrono::{DateTime, Days, Local};

use super::consts::{FINISHED, NUMBERS};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Poll {
    pub event_name: String,
    pub host: serenity::model::prelude::UserId,
    pub end_date: DateTime<Local>,
    pub start_date: DateTime<Local>,
}

impl Poll {
    fn welcome_line(&self) -> String {
        let name = self.event_name.clone();
        format!("Welcome to the scheduling thread for {name}!")
    }

    fn event_name_from_line(line: &str) -> String {
        use regex::Regex;

        let re = Regex::new(r"Welcome to the scheduling thread for (?<n>.+)!$").unwrap();
        let name = re
            .captures_iter(line.trim())
            .next()
            .unwrap()
            .name("n")
            .unwrap()
            .as_str()
            .to_string();
        name
    }

    fn host_line(&self) -> String {
        let host = self.host.clone();
        format!("Host: <@{host}>")
    }

    fn host_from_line(line: &str) -> String {
        use regex::Regex;

        let re = Regex::new(r"Host: <@(?<h>.+)>$").unwrap();
        let name = re
            .captures_iter(line.trim())
            .next()
            .unwrap()
            .name("h")
            .unwrap()
            .as_str()
            .to_string();
        name
    }
}

#[derive(Debug)]
pub struct FromStringError;

impl TryFrom<String> for Poll {
    type Error = FromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut lines = value.lines();

        let mut poll = Poll::default();
        poll.event_name = Poll::event_name_from_line(lines.next().unwrap());
        poll.host = Poll::host_from_line(lines.next().unwrap()).parse().unwrap();

        Ok(poll)
    }
}

impl From<Poll> for String {
    fn from(value: Poll) -> Self {
        let end_date = value.end_date.format("%a, %b %d at %I%P");

        let mut message_str = "".to_string();
        message_str = message_str + value.welcome_line().as_str();
        message_str = message_str + "\n";
        message_str = message_str + value.host_line().as_str();
        message_str = message_str + "\n";
        message_str = message_str
            + format!(
                "            
Event voting will be open until {end_date}.
            
"
            )
            .as_str();
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

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::poll::poll::FromStringError;

    #[test]
    fn test_froms() {
        use crate::Poll;
        let poll = Poll {
            event_name: "My event!".to_string(),
            host: 123451234.into(),
            end_date: DateTime::from_timestamp_nanos(1_000_000_000).into(),
            start_date: DateTime::from_timestamp_nanos(1_000_000_001).into(),
        };
        let str: String = String::from(poll.clone());
        let poll2: Result<Poll, FromStringError> = str.try_into();
        assert_eq!(poll, poll2.unwrap());
    }
}
