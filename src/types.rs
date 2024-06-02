use std::sync::mpsc::Sender;

use serenity::all::{ChannelId, MessageId};
use tokio::sync::Mutex;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, State, Error>;

#[derive(Debug, PartialEq)]
pub enum OrpheusStatus {
    Waiting,
    Processing,
    Stopped,
}

impl OrpheusStatus {
    pub fn as_str(&self) -> &str {
        match self {
            OrpheusStatus::Waiting => "",
            OrpheusStatus::Processing => "Processing...",
            OrpheusStatus::Stopped => "Stopped",
        }
    }
}

#[derive(Debug)]
pub struct DiscordMessage {
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

#[derive(Debug)]
pub struct State {
    // We lock the state for all polls when updating one
    // This isn't ideal but it's much easier to implement
    // than per poll locking and is fine for the current scale
    pub lock: Mutex<()>,
    pub status: Mutex<OrpheusStatus>,
    pub tx: Sender<DiscordMessage>,
}
