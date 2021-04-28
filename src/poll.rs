use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Poll {
    /// All the current answers
    pub answers: Vec<Answer>,
    /// Unique id for this poll
    pub id: String,
    /// When did this poll start?
    pub start_date: DateTime<Utc>,
    /// The poll question
    pub title: String,
    /// Forum topic info
    pub topic: Option<crate::Topic>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    /// Unique ID
    pub id: String,
    /// Option text
    pub text: String,
    /// How many people mashed this one
    pub vote_count: u32,
}


