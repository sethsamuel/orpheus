pub mod consts;
pub mod strings;

use ::serenity::all::{CacheHttp, Context, Reaction};
use ::serenity::{
    all::{AutoArchiveDuration, CreateMessage, EditThread, Http, Message, ReactionType},
    futures::future::join_all,
};
use base64::Engine;
use chrono::{Datelike, Days, NaiveDate, NaiveTime};
use consts::{FINISHED, NUMBERS};
use poise::{serenity_prelude as serenity, CreateReply, ReplyHandle};
use serde::{Deserialize, Serialize};
use serenity::all::{ChannelId, MessageId, UserId};
use std::collections::{HashMap, HashSet};
use strings::strip_zero_padding;

use crate::discord::{self, thread};
use crate::types::{Error, State};

use self::consts::NumberEmojis;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Poll {
    pub event_name: String,
    pub host: serenity::model::prelude::UserId,
    pub end_date: NaiveDate,
    pub start_date: NaiveDate,
    pub required_users: HashSet<UserId>,
    pub allowed_truants: usize,
    #[serde(skip)]
    pub eliminated_days: Vec<NumberEmojis>,
}

#[derive(Debug)]
pub struct FromStringError;

impl TryFrom<String> for Poll {
    type Error = FromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lines = value.lines();

        let line = lines.last().unwrap().trim().replace('|', "");
        let base = base64::prelude::BASE64_STANDARD.decode(line).unwrap();
        let str = String::from_utf8(base).unwrap();

        serde_json::from_str(&str).map_err(|_| FromStringError)
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
        if value.allowed_truants > 0 {
            message_str += value.allowed_truants_line().as_str();
            message_str += "\n";
            message_str += "\n";
        }
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

pub struct ReplyContext<'a> {
    pub handle: &'a ReplyHandle<'a>,
    pub ctx: &'a poise::Context<'a, State, Error>,
}

impl Poll {
    pub async fn update_days(
        &mut self,
        http: &Http,
        bot_id: UserId,
        channel_id: ChannelId,
        message_id: MessageId,
        reply: Option<ReplyContext<'_>>,
    ) -> Result<(), UpdateDaysError> {
        if let Some(reply) = &reply {
            _ = reply
                .handle
                .edit(
                    reply.ctx.to_owned(),
                    CreateReply::default().content("Checking which attendees have responded..."),
                )
                .await;
        }
        let complete_users = self
            .get_finished_users(http, channel_id, message_id, bot_id)
            .await;
        if let Some(reply) = &reply {
            _ = reply
                .handle
                .edit(
                    reply.ctx.to_owned(),
                    CreateReply::default().content("Getting availability replies..."),
                )
                .await;
        }

        let user_reactions = self
            .get_user_reactions(http, channel_id, message_id, &complete_users)
            .await;
        self.update_days_with_users(&complete_users, user_reactions)
    }

    #[tracing::instrument]
    pub fn update_days_with_users(
        &mut self,
        complete_users: &HashSet<UserId>,
        user_reactions: HashMap<NumberEmojis, Vec<UserId>>,
    ) -> Result<(), UpdateDaysError> {
        if complete_users.is_empty() {
            self.eliminated_days.clear();
            return Ok(());
        }

        let mut day_counts: HashMap<&NumberEmojis, usize> = HashMap::new();
        let mut eliminated_days: HashSet<&NumberEmojis> = HashSet::new();
        let mut required_users: HashSet<UserId> = HashSet::new();
        required_users.insert(self.host);
        let poll_required_users = self.required_users.clone();
        poll_required_users
            .iter()
            .all(|u| required_users.insert(*u));

        let complete_required_users = required_users
            .intersection(complete_users)
            .collect::<HashSet<&UserId>>();

        for n in NUMBERS.iter() {
            day_counts.insert(n, 0);
            user_reactions
                .get(n)
                .unwrap_or(&vec![])
                .iter()
                .for_each(|_| {
                    day_counts.entry(n).and_modify(|v| *v += 1);
                });
            if user_reactions
                .get(n)
                .unwrap_or(&vec![])
                .iter()
                .filter(|u| required_users.contains(u))
                .count()
                < complete_required_users
                    .len()
                    .saturating_sub(self.allowed_truants)
            {
                eliminated_days.insert(n);
            }

            if complete_required_users.contains(&self.host) {
                // Host is always required
                if !user_reactions
                    .get(n)
                    .unwrap_or(&vec![])
                    .contains(&self.host)
                {
                    eliminated_days.insert(n);
                }
            }
        }
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

    pub async fn get_user_reactions(
        &self,
        http: &Http,
        channel_id: ChannelId,
        message_id: MessageId,
        complete_users: &HashSet<UserId>,
    ) -> HashMap<NumberEmojis, Vec<UserId>> {
        let mut users_map: HashMap<NumberEmojis, Vec<UserId>> = HashMap::new();

        let rs = NUMBERS.iter();
        let fs: Vec<_> = rs
            .map(move |r| {
                discord::get_reaction_users(http, channel_id, message_id, r.as_str().to_string())
            })
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
        users_map
    }
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
        _ = thread::update(http, channel_id, message_id, self.clone()).await;

        Ok(())
    }
}
impl Poll {
    pub async fn start_thread(
        &self,
        http: &Http,
        channel_id: ChannelId,
    ) -> Result<(ChannelId, MessageId), ::serenity::Error> {
        let thread_name = format!(
            "({}-{}) {}",
            &self.start_date.format("%-m/%-d"),
            &self
                .start_date
                .checked_add_days(Days::new(6))
                .unwrap()
                .format("%-m/%-d"),
            &self.event_name,
        );
        let (c, message_id) = thread::create(http, channel_id, &thread_name, self.clone()).await;

        for n in NUMBERS {
            http.create_reaction(
                c.id,
                message_id,
                &serenity::all::ReactionType::Unicode(n.as_str().to_string()),
            )
            .await
            .inspect_err(|e| println!("Failed to add emoji! {:?}", e))?;
        }
        http.create_reaction(
            c.id,
            message_id,
            &serenity::all::ReactionType::Unicode(FINISHED.to_string()),
        )
        .await
        .inspect_err(|e| println!("Failed to add emoji! {:?}", e))?;
        Ok((c.id, message_id))
    }
}

impl Poll {
    #[tracing::instrument]
    pub async fn on_reaction(
        mut self,
        ctx: &Context,
        bot_id: UserId,
        reaction: &Reaction,
        message: Message,
    ) {
        _ = self
            .update_days(ctx.http(), bot_id, message.channel_id, message.id, None)
            .await;
        if self.eliminated_days.len() == NUMBERS.len() {
            println!("All days eliminated!");
            self.next_dates(ctx.http(), &message).await;
        }

        _ = self
            .update_message(ctx.http(), message.channel_id, message.id)
            .await;
    }
}

impl Poll {
    #[tracing::instrument]
    pub async fn next_dates(&self, http: &Http, message: &Message) {
        // No days left, start a new thread
        let mut new_poll = self.clone();
        new_poll.start_date = new_poll.start_date.checked_add_days(Days::new(7)).unwrap();
        new_poll.eliminated_days = vec![];
        new_poll.end_date = new_poll.start_date.checked_sub_days(Days::new(1)).unwrap();
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

        _ = http
                    .send_message(
                        message.channel_id,
                        vec![],
                        &CreateMessage::new().content(format!(
                            "All dates eliminated! Created a new thread <#{}> with next set of dates. This thread is locked and will be archived in one day.",
                            channel_id
                        )),
                    )
                    .await;

        _ = http
            .edit_thread(
                message.channel_id,
                &EditThread::new()
                    .locked(true)
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
    use std::collections::{HashMap, HashSet};

    use chrono::NaiveDate;
    use serenity::all::UserId;

    use crate::poll::consts::NumberEmojis;

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

    #[test]
    fn test_update_days_with_users() {
        use super::Poll;
        let mut poll = Poll {
            event_name: "My event!".to_string(),
            host: 123451234.into(),
            end_date: NaiveDate::from_ymd_opt(2024, 2, 11).unwrap(),
            start_date: NaiveDate::from_ymd_opt(2024, 2, 18).unwrap(),
            allowed_truants: 1,
            ..Default::default()
        };

        let user_1 = UserId::new(1);
        let user_2 = UserId::new(2);

        let mut complete_users = HashSet::new();
        complete_users.insert(poll.host);

        let mut user_reactions = HashMap::new();
        user_reactions.insert(NumberEmojis::One, vec![poll.host, user_1]);
        user_reactions.insert(NumberEmojis::Two, vec![]);
        user_reactions.insert(NumberEmojis::Three, vec![user_1, user_2]);
        user_reactions.insert(NumberEmojis::Four, vec![poll.host, user_2]);
        user_reactions.insert(NumberEmojis::Five, vec![poll.host, user_1, user_2]);

        let result = poll.update_days_with_users(&complete_users, user_reactions);
        assert!(result.is_ok());

        // Host trumps day 3
        assert!(!poll.eliminated_days.contains(&NumberEmojis::One));
        assert!(poll.eliminated_days.contains(&NumberEmojis::Two));
        assert!(poll.eliminated_days.contains(&NumberEmojis::Three));
        assert!(!poll.eliminated_days.contains(&NumberEmojis::Four));
        assert!(!poll.eliminated_days.contains(&NumberEmojis::Five));
    }
}
