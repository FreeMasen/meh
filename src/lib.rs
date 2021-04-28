//! Meh!
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
mod deal;
mod poll;
mod video;

pub use deal::Deal;
pub use poll::Poll;
pub use video::Video;

#[derive(Debug, Deserialize, Serialize)]
/// The response from a call to the meh api endpoint
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
    pub created_at: DateTime<Utc>,
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
