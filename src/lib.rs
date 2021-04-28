use serde::{Deserialize, Serialize};
mod deal;
mod poll;
mod video;

pub use deal::Deal;
pub use poll::Poll;
pub use video::Video;

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub deal: Deal,
    #[serde(default)] 
    pub poll: Option<Poll>,
    #[serde(default)] 
    pub video: Option<Video>,
}

pub fn construct_url(api_key: &str) -> String {
    format!("https://api.meh.com/1/current.json?apikey={}", api_key)
}
