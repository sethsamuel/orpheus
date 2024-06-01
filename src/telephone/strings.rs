use crate::poll::consts::FINISHED;

use super::Telephone;

impl Telephone {
    pub fn welcome_line(&self) -> String {
        format!("Welcome to Narrative Telephone!")
    }

    pub fn host(&self) -> String {
        format!("Host: <@{}>", self.host)
    }

    pub fn players(&self) -> String {
        format!(
            "Players:\n{}",
            self.players
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
                .join("\n")
        )
    }

    pub fn folder(&self) -> String {
        format!("[{}](Shared folder)", self.folder_url)
    }
}
