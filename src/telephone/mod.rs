use std::collections::HashSet;

use base64::Engine;
use serde::{Deserialize, Serialize};
use serenity::all::{CacheHttp, Context, CreateMessage, Message, Reaction, UserId};

use crate::discord::{self, thread};
use rand::seq::SliceRandom;

use self::consts::{FINISHED, START, STORY_TELLER};

pub mod consts;
pub mod strings;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Telephone {
    pub host: serenity::model::prelude::UserId,
    pub folder_url: String,
    pub lead: Option<UserId>,
    pub players: Vec<UserId>,
    pub finished_players: HashSet<UserId>,
    pub nag_interval: u8,
}

#[derive(Debug)]
pub struct FromStringError;

impl TryFrom<String> for Telephone {
    type Error = FromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lines = value.lines();

        let line = lines.last().unwrap().trim().replace('|', "");
        let base = base64::prelude::BASE64_STANDARD.decode(line).unwrap();
        let str = String::from_utf8(base).unwrap();
        let telephone = serde_json::from_str(&str).unwrap();

        Ok(telephone)
    }
}

impl From<Telephone> for String {
    fn from(value: Telephone) -> Self {
        let mut message_str = "".to_string();

        message_str += value.welcome_line().as_str();
        message_str += "\n\n";

        message_str += value.players().as_str();
        message_str += "\n\n";

        message_str += value.folder().as_str();
        message_str += "\n\n";

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

impl Telephone {
    pub fn set_lead(&mut self) {
        if let Some(lead_id) = self.lead {
            self.players = self
                .players
                .iter()
                .filter(|id| **id != lead_id)
                .cloned()
                .collect::<Vec<UserId>>();
            self.players.insert(0, lead_id);
        }
    }

    pub fn next_player_id(&self) -> Option<&UserId> {
        self.players
            .iter()
            .find(|id| !self.finished_players.contains(id))
    }
}

impl Telephone {
    #[tracing::instrument]
    pub async fn on_reaction(
        mut self,
        ctx: &Context,
        bot_id: UserId,
        reaction: &Reaction,
        message: Message,
    ) {
        match reaction.emoji.to_string().as_str() {
            STORY_TELLER => {
                let users = discord::get_reaction_users(
                    ctx.http(),
                    message.channel_id,
                    message.id,
                    FINISHED.to_string(),
                )
                .await
                .unwrap()
                .1;
                let mut user_ids: Vec<UserId> = users.iter().map(|u| u.id).collect();
                user_ids.shuffle(&mut rand::thread_rng());
                self.players = user_ids;
                self.set_lead();
            }
            FINISHED => {
                let user_id = reaction.user_id.unwrap();
                self.finished_players.insert(user_id);

                let next_player_id = self.next_player_id();
                if let Some(next_player_id) = next_player_id {
                    _ = ctx
                    .http()
                    .send_message(
                        reaction.channel_id,
                        vec![],
                        &CreateMessage::new()
                            .content(format!("<@{}> you're up! Watch the latest video, then immediately record your version. Upload it to the link in the thread message and name the file `{}[Your name]`.\n\nOnce you're done, react to the thread message with {}.", next_player_id, self.players.iter().position(|id| id == next_player_id).unwrap(), FINISHED)),
                    )
                    .await;
                } else {
                    _ = ctx
                    .http()
                    .send_message(
                        reaction.channel_id,
                        vec![],
                        &CreateMessage::new()
                            .content("We're all done! Go ahead and use `\\orpheus save_me` in the main channel to schedule a watch party!"),
                    )
                    .await;
                }
            }
            START => {
                if reaction.user_id.unwrap() != self.host {
                    _ = ctx
                        .http()
                        .send_message(
                            reaction.channel_id,
                            vec![],
                            &CreateMessage::new().content(format!(
                                "Sorry <@{}>, only the host can start the story time",
                                reaction.user_id.unwrap()
                            )),
                        )
                        .await;
                    return;
                }
                if self.lead.is_none() {
                    self.lead = Some(*self.players.first().unwrap());
                    self.set_lead();
                }
                _ = ctx
                    .http()
                    .send_message(
                        reaction.channel_id,
                        vec![],
                        &CreateMessage::new()
                            .content(format!("<@{}> you're up! Record your story (it should be about two minutes), then upload it to the link in the thread message. You should name the file `0[Your name]`.\n\nOnce you're done, react to the thread message with {}.", self.lead.unwrap(), FINISHED)),
                    )
                    .await;
            }
            _ => println!(
                "Unknown reaction for story {:?}",
                reaction.emoji.to_string()
            ),
        }

        _ = thread::update(ctx.http(), reaction.channel_id, reaction.message_id, self).await
    }
}
