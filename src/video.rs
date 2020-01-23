use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: String,
    pub start_date: DateTime<Utc>,
    pub title: String,
    pub url: String,
    pub topic: Topic,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub comment_count: u32,
    pub created_at: DateTime<Utc>,
    pub id: String,
    pub reply_count: u32,
    pub url: String,
    pub vote_count: u32,
}
