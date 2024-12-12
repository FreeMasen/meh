use time::OffsetDateTime;
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
/// Description of the current deal on meh.com
pub struct Deal {
    /// The bulleted list that appears below the item's name
    pub features: String,
    /// Unique id for this deal
    pub id: String,
    /// The items for sale
    pub items: Vec<Item>,
    #[serde(default)]
    /// details about the sale (currently just when it sold out)
    pub launches: Vec<Launch>,
    /// List of CDN urls for all of the images
    pub photos: Vec<String>,
    /// minimum/maximum limits for this purchase
    pub purchase_quantity: PurchaseQuantity,
    /// The item's name
    pub title: String,
    /// Bulleted list of additional information
    pub specifications: String,
    /// A snarky story for your enjoyment
    pub story: Story,
    /// The current website theme
    pub theme: Theme,
    /// The direct url to this item
    pub url: String,
    /// When this deal will expire
    #[serde(
        with = "time::serde::rfc3339::option",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub end_date: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
/// A single item in the current deal
pub struct Item {
    /// What makes this item unique?
    pub attributes: Vec<Attribute>,
    /// New, Used, Refurbished...
    pub condition: String,
    /// Unique id for this item
    pub id: String,
    /// The price they are selling it for
    pub price: f32,
    /// The main photo url for this variation
    pub photo: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Launch {
    /// If this sold out... when?
    #[serde(with = "time::serde::rfc3339::option",)]
    pub sold_out_at: Option<OffsetDateTime>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseQuantity {
    pub maximum_limit: u32,
    pub minimum_limit: u32,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Story {
    pub title: String,
    pub body: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    pub accent_color: String,
    pub background_color: String,
    pub background_image: String,
    pub foreground: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}
