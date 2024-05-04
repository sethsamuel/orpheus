use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime};

use super::poll::Poll;

const END_DATE_FORMAT: &str = "%a, %b %d at %I%P";

pub fn strip_zero_padding(str: &str) -> String {
    let remove_zeros = regex::Regex::new(r"0(\d)").unwrap();
    let clean = remove_zeros.replace_all(str, "$1");
    clean.to_string()
}

impl Poll {
    pub fn welcome_line(&self) -> String {
        let name = self.event_name.clone();
        format!("Welcome to the scheduling thread for **{name}**!")
    }

    pub fn host_line(&self) -> String {
        let host = self.host.clone();
        format!("Host: <@{host}>")
    }

    pub fn ends_at_line(&self) -> String {
        let end_time =
            NaiveDateTime::new(self.end_date, NaiveTime::from_hms_opt(22, 0, 0).unwrap());
        let end_date = end_time.format(END_DATE_FORMAT).to_string();
        let clean = strip_zero_padding(&end_date);

        format!("Event voting will be open until {clean}")
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::poll::poll::Poll;

    #[test]
    fn test_ends_at_line() {
        let mut poll = Poll::default();
        poll.end_date = NaiveDate::from_ymd_opt(2024, 03, 4).unwrap();
        let line = poll.ends_at_line();
        assert_eq!(line, "Event voting will be open until Mon, Mar 4 at 10pm")
    }
}