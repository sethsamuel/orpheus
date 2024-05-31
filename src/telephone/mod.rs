use base64::Engine;
use serde::{Deserialize, Serialize};
use serenity::all::UserId;

pub mod consts;

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Telephone {
    pub host: serenity::model::prelude::UserId,
    pub lead: Option<UserId>,
    pub players: Vec<UserId>,
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

        message_str = message_str
            + "||"
            + base64::prelude::BASE64_STANDARD
                .encode(serde_json::to_string(&value).unwrap().as_bytes())
                .as_str()
            + "||";

        message_str
    }
}
