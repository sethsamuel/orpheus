use std::sync::atomic::AtomicU32;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, State, Error>;

pub struct State {
    pub poise_mentions: AtomicU32,
}
