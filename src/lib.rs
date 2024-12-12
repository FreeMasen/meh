//! Meh!
use time::OffsetDateTime;
use serde::{Deserialize, Serialize};
mod deal;
mod poll;
mod video;

pub use deal::Deal;
pub use poll::Poll;
pub use video::Video;

/// The response from a call to the meh api endpoint
#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    /// The current deal
    pub deal: Deal,
    #[serde(default)] 
    /// The current poll
    pub poll: Option<Poll>,
    #[serde(default)] 
    /// The current video
    pub video: Option<Video>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    /// How many people commented on it?
    pub comment_count: u32,
    /// When was this created?
    #[serde(with = "time::serde::rfc3339",)]
    pub created_at: OffsetDateTime,
    /// Unique ID
    pub id: String,
    /// How many replies to it?
    pub reply_count: u32,
    /// Where can you find it?
    pub url: String,
    /// Total vote count
    pub vote_count: u32,
}

/// Build a meh api url using the provided `api_key`
pub fn construct_url(api_key: &str) -> String {
    format!("https://api.meh.com/1/current.json?apikey={}", api_key)
}
