use tokio::sync::Mutex;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, State, Error>;

#[derive(Debug)]
pub enum OrpheusStatus {
    Waiting,
    Processing,
}

#[derive(Debug)]
pub struct State {
    // We lock the state for all polls when updating one
    // This isn't ideal but it's much easier to implement
    // than per poll locking and is fine for the current scale
    pub lock: Mutex<()>,
    pub status: Mutex<OrpheusStatus>,
}
