use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    /// Unique ID
    pub id: String,
    /// When did it start?
    #[serde(with = "time::serde::rfc3339")]
    pub start_date: OffsetDateTime,
    /// What is it called?
    pub title: String,
    /// Where can I find it?
    pub url: String,
    /// Forum Topic Information
    pub topic: crate::Topic,
}
