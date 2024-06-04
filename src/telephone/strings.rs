use crate::{
    poll::consts::FINISHED,
    telephone::consts::{START, STORY_TELLER},
};

use super::Telephone;

impl Telephone {
    pub fn welcome_line(&self) -> String {
        "Welcome to Narrative Telephone!".to_string()
    }

    pub fn host(&self) -> String {
        format!("Host: <@{}> (hit {} to start)", self.host, START)
    }

    pub fn players(&self) -> String {
        format!(
            "Players:\n{}\n\nClick {} to signup as a player",
            match self.players.len() {
                0 => "No one yet 😥".to_string(),
                _ => self
                    .players
                    .iter()
                    .map(|id| format!(
                        "<@{}> {}",
                        id,
                        match self.finished_players.contains(id) {
                            true => FINISHED,
                            false => "",
                        }
                    ))
                    .collect::<Vec<String>>()
                    .join("\n"),
            },
            STORY_TELLER
        )
    }

    pub fn folder(&self) -> String {
        format!("[Shared folder]({})", self.folder_url)
    }
}
