use chrono::{DateTime, Utc};
use serde::{self, Deserialize};

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemSeller {
    pub username: String,
    #[serde(rename = "feedbackPercentage")]
    pub feedback_percentage: String,
    #[serde(rename = "feedbackScore")]
    pub feedback_score: u64,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemPrice {
    pub value: String,
    pub currency: String, // TODO: change to enum ?
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemBidPrice {
    pub value: String,
    pub currency: String, // TODO: change to enum ?
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemImage {
    #[serde(rename = "imageUrl")]
    pub image_url: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemSummary {
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub title: String,
    #[serde(rename = "itemWebUrl")]
    pub item_web_url: String,
    pub price: Option<ItemPrice>,
    #[serde(rename = "currentBidPrice")]
    pub current_bid_price: Option<ItemBidPrice>,
    pub condition: Option<String>,
    #[serde(rename = "buyingOptions")]
    pub buying_options: Vec<String>,
    pub image: Option<ItemImage>,
    #[serde(rename = "itemEndDate")]
    pub item_end_date: Option<String>,
    pub seller: ItemSeller,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
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

    pub fn id(&self) -> Option<&str> {
        self.item_id.split("|").skip(1).next()
    }

    pub fn bin_price(&self) -> Option<(f64, &str)> {
        self.price.as_ref().map(|p| {
            (
                p.value.parse().expect("couldn't parse bin price"),
                p.currency.as_str(),
            )
        })
    }

    pub fn current_bid_price(&self) -> Option<(f64, &str)> {
        self.current_bid_price.as_ref().map(|p| {
            (
                p.value.parse().expect("couldn't parse current bid price"),
                p.currency.as_str(),
            )
        })
    }

    pub fn buying_options(&self) -> Vec<String> {
        self.buying_options
            .iter()
            .map(|b| match b.as_str() {
                "FIXED_PRICE" => "BIN".to_owned(),
                "AUCTION" => "BID".to_owned(),
                "BEST_OFFER" => "OFFER".to_owned(),
                x => x.to_owned(),
            })
            .collect()
    }
}
