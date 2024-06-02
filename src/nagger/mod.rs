use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::Arc,
};

use chrono::{DateTime, Local, Utc};
use serenity::all::{ChannelId, CreateMessage, Http, Message, MessageId};

use crate::{discord::thread, telephone::Telephone, types::DiscordMessage};

#[derive(Debug)]
pub struct Nagger {
    pub messages: HashSet<DiscordMessage>,
    pub nagged_at: HashMap<DiscordMessage, DateTime<Local>>,
    pub http: Option<Arc<Http>>,
}

impl Nagger {
    pub fn new() -> Nagger {
        Nagger {
            messages: HashSet::new(),
            nagged_at: HashMap::new(),
            http: None,
        }
    }
    pub async fn init(&mut self, http: Arc<Http>) {
        self.http = Some(http.clone());

        self.messages = HashSet::new();
        // Read file and init
        if let Ok(contents) = fs::read_to_string("/tmp/orpheus") {
            let lines = contents.lines();
            for line in lines {
                let parts: Vec<&str> = line.split(',').collect();
                let discord_message = DiscordMessage {
                    channel_id: ChannelId::new(parts[0].parse::<u64>().unwrap()),
                    message_id: MessageId::new(parts[1].parse::<u64>().unwrap()),
                };
                self.update(discord_message).await;
            }
        } else {
            println!("No saved state file")
        }
        self.save();
    }

    pub fn save(&self) {
        let str = self
            .messages
            .iter()
            .map(|m| format!("{},{}", m.channel_id, m.message_id))
            .collect::<Vec<String>>()
            .join(",");

        match fs::write("/tmp/orpheus", str) {
            Ok(_) => println!("Wrote state to file"),
            Err(e) => println!("Error writing state to file {:?}", e),
        }
    }
}

impl Nagger {
    pub async fn update(&mut self, from_message: DiscordMessage) -> Option<Message> {
        self.http.as_ref()?;
        let http = self.http.as_ref().unwrap();
        if let Ok(message) = http
            .get_message(from_message.channel_id, from_message.message_id)
            .await
        {
            if thread::is_locked(http, message.channel_id).await {
                return None;
            }
            if let Some(telephone) = thread::try_from::<Telephone>(&message) {
                self.messages.insert(from_message);
                self.nagged_at.insert(from_message, telephone.nagged_at);
            }
            return Some(message);
        }
        None
    }

    pub async fn execute(&mut self) {
        for discord_message in self.messages.clone().iter() {
            if Utc::now().timestamp_millis()
                - self
                    .nagged_at
                    .get(discord_message)
                    .unwrap_or(&chrono::offset::Local::now())
                    .timestamp_millis()
                > 1000 * 60 * 60 * 7
            {
                continue;
            }
            self.check_message(*discord_message).await;
        }
    }

    pub async fn check_message(&mut self, discord_message: DiscordMessage) {
        if let Some(message) = self.update(discord_message).await {
            if let Some(telephone) = thread::try_from::<Telephone>(&message) {
                if Utc::now().timestamp_millis() - telephone.nagged_at.timestamp_millis()
                    > 1000 * 60 * 60 * 7
                {
                    self.nag(message, telephone).await;
                }
            }
        }
    }
}

impl Nagger {
    pub async fn nag(&mut self, message: Message, mut telephone: Telephone) {
        let user_id = telephone
            .players
            .iter()
            .find(|id| !telephone.finished_players.contains(id));
        if user_id.is_none() {
            return;
        }
        _ = self
            .http
            .clone()
            .unwrap()
            .send_message(
                message.channel_id,
                vec![],
                &CreateMessage::new().content(format!(
                "Psst, <@{}> {}",
                user_id.unwrap(),
                "https://giphy.com/gifs/siliconvalleyhbo-watching-goodbye-window-26BRuo6sLetdllPAQ"
            )),
            )
            .await;
        telephone.nagged_at = Utc::now().into();
        _ = thread::update(
            &self.http.clone().unwrap(),
            message.channel_id,
            message.id,
            telephone,
        )
        .await;
    }
}
