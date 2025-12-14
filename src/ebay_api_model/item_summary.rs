use serde::{self, Deserialize};

#[derive(Deserialize, Debug)]
pub struct ItemSummary {
    #[serde(rename = "itemId")]
    pub item_id: String,
    pub title: String,
}

#[derive(Deserialize, Debug)]
pub struct ItemSummaryResponse {
    #[serde(rename = "itemSummaries")]
    pub item_summaries: Vec<ItemSummary>,
}
