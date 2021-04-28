use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    /// Unique ID
    pub id: String,
    /// When did it start?
    pub start_date: DateTime<Utc>,
    /// What is it called?
    pub title: String,
    /// Where can I find it?
    pub url: String,
    /// Forum Topic Information
    pub topic: crate::Topic,
}

