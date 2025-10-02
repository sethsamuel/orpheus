use chrono::{NaiveDateTime, NaiveTime};

use super::Poll;

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
        let host = self.host;
        format!("Host: <@{host}>")
    }

    pub fn required_users_line(&self) -> String {
        let mut line = "Required attendees:".to_string();
        for u in self.required_users.clone().iter() {
            line += format!(" <@{}>", u).as_str();
        }
        line.to_string()
    }

    pub fn allowed_truants_line(&self) -> String {
        format!("Allowed truancy: {}", self.allowed_truants)
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

    use crate::poll::Poll;

    #[test]
    fn test_ends_at_line() {
        let poll = Poll {
            end_date: NaiveDate::from_ymd_opt(2024, 3, 4).unwrap(),
            ..Default::default()
        };
        let line = poll.ends_at_line();
        assert_eq!(line, "Event voting will be open until Mon, Mar 4 at 10pm")
    }
}
