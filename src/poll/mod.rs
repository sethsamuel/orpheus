pub mod consts;
pub mod strings;

use ::serenity::{
    all::{
        AutoArchiveDuration, ChannelType, CreateMessage, EditThread, Http, Message, ReactionType,
    },
    futures::future::join_all,
};
use base64::Engine;
use chrono::{Datelike, Days, NaiveDate, NaiveTime, Utc};
use consts::{FINISHED, NUMBERS};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, EditMessage, MessageId, UserId};
use std::collections::{HashMap, HashSet};
use strings::strip_zero_padding;

use self::consts::NumberEmojis;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Poll {
    pub event_name: String,
    pub host: serenity::model::prelude::UserId,
    pub end_date: NaiveDate,
    pub start_date: NaiveDate,
    pub required_users: Option<Vec<UserId>>,
    #[serde(skip)]
    pub eliminated_days: Vec<NumberEmojis>,
}

#[derive(Debug)]
pub struct FromStringError;

impl TryFrom<String> for Poll {
    type Error = FromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lines = value.lines();

        // let mut poll = Poll::default();
        // poll.event_name = Poll::event_name_from_line(lines.next().unwrap());
        // poll.host = Poll::host_from_line(lines.next().unwrap()).parse().unwrap();
        // poll.end_date = Poll::end_date_from_line(lines.next().unwrap());

        let line = lines.last().unwrap().trim().replace('|', "");
        let base = base64::prelude::BASE64_STANDARD.decode(line).unwrap();
        let str = String::from_utf8(base).unwrap();
        let poll = serde_json::from_str(&str).unwrap();

        Ok(poll)
    }
}

impl From<Poll> for String {
    fn from(value: Poll) -> Self {
        let mut message_str = "".to_string();
        message_str += value.welcome_line().as_str();
        message_str += "\n";
        message_str += "\n";
        message_str += value.host_line().as_str();
        message_str += "\n";
        message_str += "\n";
        message_str += value.required_users_line().as_str();
        message_str += "\n";
        message_str += "\n";
        message_str += value.ends_at_line().as_str();
        message_str += "\n";
        message_str += "\n";

        let date_format = "%a, %b %d at %I%P";

        for (i, number) in NUMBERS.iter().enumerate() {
            let emoji = number.as_str();
            let date = value
                .start_date
                .checked_add_days(Days::new(i.try_into().unwrap()))
                .unwrap();
            let hour = match date.weekday() {
                chrono::Weekday::Sat => 15,
                chrono::Weekday::Sun => 15,
                _ => 19,
            };
            let date_time = date
                .and_time(NaiveTime::from_hms_opt(hour, 0, 0).unwrap())
                .format(date_format)
                .to_string();

            let clean = strip_zero_padding(&date_time);
            let mut eliminated = "";
            if value.eliminated_days.contains(number) {
                eliminated = "~~";
            }
            message_str += format!("{eliminated}{emoji} {clean}{eliminated}\n").as_str();
        }
        message_str += format!("\nTo lock in your availability, hit {FINISHED}. If you need to change your answers after, please uncheck then recheck {FINISHED}.").as_str();
        message_str += "\n";
        message_str +=
            format!("\nIf you're sure you can't make any days, just hit {FINISHED}.").as_str();
        message_str += "\n";
        message_str += "\n";
        // message_str += format!("Orpehus is {}");
        // message_str += "\n";
        // message_str += "\n";

        message_str += "Orpehus Magic String (feel free to ignore):";
        message_str += "\n";
        message_str = message_str
            + "||"
            + base64::prelude::BASE64_STANDARD
                .encode(serde_json::to_string(&value).unwrap().as_bytes())
                .as_str()
            + "||";

        message_str
    }
}

pub struct UpdateDaysError;

impl Poll {
    #[tracing::instrument]
    pub async fn update_days(
        &mut self,
        http: &Http,
        bot_id: UserId,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<(), UpdateDaysError> {
        let complete_users = self
            .get_finished_users(http, channel_id, message_id, bot_id)
            .await;

        if complete_users.is_empty() {
            self.eliminated_days.clear();
            return Ok(());
        }
        let mut users_map: HashMap<NumberEmojis, Vec<UserId>> = HashMap::new();

        let rs = NUMBERS
            .iter()
            .map(|n| ReactionType::Unicode(n.as_str().to_string()));
        let fs: Vec<_> = rs
            .map(move |r| get_reaction_users(http, channel_id, message_id, r))
            .collect();

        let user_reactions = join_all(fs).await;
        for reactions in user_reactions {
            let r = reactions.unwrap();
            let n = r.0.to_string().as_str().try_into().unwrap();
            let users = r.1;
            users_map.insert(
                n,
                users
                    .into_iter()
                    .map(|u| u.id)
                    .filter(|id| complete_users.contains(id))
                    .collect(),
            );
        }

        let mut day_counts: HashMap<&NumberEmojis, usize> = HashMap::new();
        let mut eliminated_days: HashSet<&NumberEmojis> = HashSet::new();
        let mut required_users: HashSet<UserId> = HashSet::new();
        required_users.insert(self.host);
        let poll_required_users = self.required_users.clone().unwrap_or_default();
        poll_required_users
            .iter()
            .all(|u| required_users.insert(*u));

        let complete_required_users = required_users
            .intersection(&complete_users)
            .collect::<HashSet<&UserId>>();

        for n in NUMBERS.iter() {
            day_counts.insert(n, 0);
            users_map.get(n).unwrap_or(&vec![]).iter().for_each(|_| {
                day_counts.entry(n).and_modify(|v| *v += 1);
            });
            println!("{:?}", n);
            println!("{:?}", users_map);
            if users_map
                .get(n)
                .unwrap_or(&vec![])
                .iter()
                .filter(|u| required_users.contains(u))
                .count()
                != complete_required_users.len()
            {
                eliminated_days.insert(n);
            }
        }
        println!("{:?}", day_counts);
        self.eliminated_days = eliminated_days.iter().map(|n| **n).collect();

        Ok(())
    }

    pub async fn get_finished_users(
        &self,
        http: &Http,
        channel_id: ChannelId,
        message_id: MessageId,
        bot_id: UserId,
    ) -> HashSet<UserId> {
        http.get_reaction_users(
            channel_id,
            message_id,
            &ReactionType::Unicode(FINISHED.to_string()),
            100,
            None,
        )
        .await
        .unwrap()
        .into_iter()
        .map(|u| u.id)
        .filter(|id| *id != bot_id)
        .collect::<HashSet<UserId>>()
    }
}

#[tracing::instrument]
async fn get_reaction_users(
    http: &Http,
    channel_id: ChannelId,
    message_id: MessageId,
    r: ReactionType,
) -> Result<(ReactionType, Vec<serenity::User>), ::serenity::Error> {
    let users = http
        .get_reaction_users(channel_id, message_id, &r, 50, None)
        .await?;
    Ok((r, users))
}

pub struct UpdateError;
impl Poll {
    #[tracing::instrument]
    pub async fn update_message(
        &self,
        http: &Http,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<(), UpdateError> {
        let new_content: String = self.clone().into();
        let _ = http
            .edit_message(
                channel_id,
                message_id,
                &EditMessage::new().content(new_content),
                vec![],
            )
            .await;

        Ok(())
    }
}
impl Poll {
    pub async fn start_thread(
        &self,
        http: &Http,
        channel_id: ChannelId,
    ) -> Result<(ChannelId, MessageId), ::serenity::Error> {
        let c = http
            .create_thread(
                channel_id,
                &serenity::builder::CreateThread::new(self.event_name.to_string())
                    .kind(ChannelType::PublicThread),
                Some("Time to schedule"),
            )
            .await
            .unwrap();

        let message_str: String = self.clone().into();

        let message = c.id.say(http, message_str).await;
        let message_id = message.unwrap().id;

        for n in NUMBERS {
            let _ = http
                .create_reaction(
                    c.id,
                    message_id,
                    &serenity::all::ReactionType::Unicode(n.as_str().to_string()),
                )
                .await
                .inspect_err(|e| println!("Failed to add emoji! {:?}", e));
        }
        let _ = http
            .create_reaction(
                c.id,
                message_id,
                &serenity::all::ReactionType::Unicode(FINISHED.to_string()),
            )
            .await
            .inspect_err(|e| println!("Failed to add emoji! {:?}", e));
        Ok((c.id, message_id))
    }
}

impl Poll {
    #[tracing::instrument]
    pub async fn next_dates(&self, http: &Http, message: &Message) {
        // No days left, start a new thread
        let mut new_poll = self.clone();
        new_poll.start_date = new_poll.start_date.checked_add_days(Days::new(7)).unwrap();
        new_poll.eliminated_days = vec![];
        new_poll.end_date = Utc::now()
            .date_naive()
            .checked_add_days(Days::new(1))
            .unwrap();
        let (channel_id, _) = new_poll
            .start_thread(
                http,
                message
                    .channel(http)
                    .await
                    .unwrap()
                    .guild()
                    .unwrap()
                    .parent_id
                    .unwrap(),
            )
            .await
            .unwrap();

        let _ = http
                    .send_message(
                        message.channel_id,
                        vec![],
                        &CreateMessage::new().content(format!(
                            "All dates eliminated! Created a new thread <#{}> with next set of dates. This thread is locked and will be archived in one day.",
                            channel_id
                        )),
                    )
                    .await;

        let _ = http
            .edit_thread(
                message.channel_id,
                &EditThread::new()
                    .locked(true)
                    .archived(true)
                    .auto_archive_duration(AutoArchiveDuration::OneDay),
                Some("Voting closed"),
            )
            .await
            .inspect_err(|e| println!("Error closing thread {}", e))
            .inspect(|_| println!("Channel archived"));
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::FromStringError;

    #[test]
    fn test_froms() {
        use super::Poll;
        let poll = Poll {
            event_name: "My event!".to_string(),
            host: 123451234.into(),
            end_date: NaiveDate::from_ymd_opt(2024, 2, 11).unwrap(),
            start_date: NaiveDate::from_ymd_opt(2024, 2, 18).unwrap(),
            ..Default::default()
        };
        let str: String = String::from(poll.clone());
        let poll2: Result<Poll, FromStringError> = str.try_into();
        assert_eq!(poll, poll2.unwrap());
    }
}
