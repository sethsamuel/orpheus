use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::Arc,
};

use chrono::{DateTime, Local};
use serenity::all::{ChannelId, Http, MessageId};

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
    pub async fn update(&mut self, from_message: DiscordMessage) {
        if self.http.is_none() {
            return;
        }
        let http = self.http.as_ref().unwrap();
        if let Ok(message) = http
            .get_message(from_message.channel_id, from_message.message_id)
            .await
        {
            if thread::is_locked(http, message.channel_id).await {
                return;
            }
            if let Some(telephone) = thread::try_from::<Telephone>(&message) {
                self.messages.insert(from_message);
                self.nagged_at.insert(from_message, telephone.nagged_at);
            }
        }
    }
    pub async fn execute(&mut self) {
        // self.http.get
    }
}
