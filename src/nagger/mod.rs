
use std::{collections::HashSet, sync::Arc};


use serenity::all::Http;

use crate::telephone::Telephone;

#[derive(Debug)]
pub struct Nagger {
    pub telephones: HashSet<Telephone>,
    pub http: Option<Arc<Http>>,
}

impl Nagger {
    pub fn new() -> Nagger {
        Nagger {
            telephones: HashSet::new(),
            http: None,
        }
    }
    pub async fn init(&mut self, http: Arc<Http>) {
        self.http = Some(http);

        // Read file and init
    }
}

impl Nagger {
    pub async fn execute(&mut self) {
        // self.http.get
    }
}
