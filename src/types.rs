use rocket::tokio::sync::watch::Receiver;
use rocket::serde::{Serialize, Deserialize};

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct LocationConfig {
    #[serde(rename(serialize = "prefferedName", deserialize = "prefferedName"))]
    pub preffered_name: Option<String>,
    pub order: u64
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    #[serde(rename(serialize = "eventId", deserialize = "eventId"))]
    pub event_id: String,
    pub locations: HashMap<String, LocationConfig>
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Location {
    pub id: String,
    pub name: String,
    pub capacity: u64,
    pub count: u64,
    pub order: u64
}


#[derive(Debug)]
pub struct Context {
    reciever: Receiver<HashMap<String, Location>>
}

impl Context {
    pub fn new(rx: Receiver<HashMap<String, Location>>) -> Context {
        return Context{reciever: rx};
    }

    pub fn get_reciever(&self) -> Receiver<HashMap<String, Location>> {
        return self.reciever.clone();
    }
}