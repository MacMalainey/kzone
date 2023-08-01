use rocket::serde::Deserialize;

// Event Time Types

pub type JsonCurrentEventTimes = MultiEntry<IdOnly>;

// Location Types

pub type JsonLocation = SingleEntry<JsonLocData>;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct JsonLocData {
    pub id: String,
    pub attributes: JsonLocAttr
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct JsonLocAttr {
    pub name: String,
    pub max_occupancy: u64,
}

// Location Event Time Types

pub type JsonLocEventTime = SingleEntry<JsonLocEventTimeData>;
pub type JsonAllLocEventTime = MultiEntry<JsonLocEventTimeData>;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct JsonLocEventTimeData {
    pub id: String,
    pub attributes: JsonLocEventTimeAttr,
    pub relationships: Option<JsonLocEventTimeRel>
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct JsonLocEventTimeAttr {
    pub guest_count: Option<u64>,
    pub regular_count: Option<u64>
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct JsonLocEventTimeRel {
    pub location: SingleEntry<IdOnly>
}

// Misc helper types

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct MultiEntry<T> {
    pub data: Vec<T>
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct SingleEntry<T> {
    pub data: T
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct IdOnly {
    pub id: String
}