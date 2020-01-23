use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deal {
    pub features: String,
    pub id: String,
    pub items: Vec<Item>,
    #[serde(default)]
    pub launches: Vec<Launch>,
    pub photos: Vec<String>,
    pub purchase_quantity: PurchaseQuantity,
    pub title: String,
    pub specifications: String,
    pub story: Story,
    pub theme: Theme,
    pub url: String,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    pub attributes: Vec<Attribute>,
    pub condition: String,
    pub id: String,
    pub price: f32,
    pub photo: String,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Launch {
    pub sold_out_at: Option<DateTime<Utc>>,
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
