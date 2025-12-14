use chrono::{DateTime, Utc};
use serde::{self, Deserialize};

#[derive(Deserialize, Debug)]
pub struct ItemPrice {
    pub value: String,
    pub currency: String, // TODO: change to enum ?
}

#[derive(Deserialize, Debug)]
pub struct ItemImage {
    #[serde(rename = "imageUrl")]
    pub image_url: String,
}

#[derive(Deserialize, Debug)]
pub struct ItemSummary {
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub title: String,
    pub price: ItemPrice,
    pub condition: String,
    #[serde(rename = "buyingOptions")]
    pub buying_options: Vec<String>,
    pub image: ItemImage,
    #[serde(rename = "itemEndDate")]
    pub item_end_date: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ItemSummaryResponse {
    #[serde(rename = "itemSummaries")]
    pub item_summaries: Vec<ItemSummary>,
}

impl ItemSummary {
    pub fn is_auction(&self) -> bool {
        self.buying_options.iter().any(|o| o == "AUCTION")
    }

    pub fn end_timestamp(&self) -> Option<i64> {
        self.item_end_date.as_ref().map(|item_end_date| {
            item_end_date
                .parse::<DateTime<Utc>>()
                .expect("Failed to parse date")
                .timestamp()
        })
    }

    pub fn price(&self) -> f64 {
        self.price.value.parse().expect("couldn't parse price")
    }
}
