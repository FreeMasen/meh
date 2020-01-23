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
    pub poll: Poll,
    pub video: Video,
}
