use webhook::client::WebhookClient;

use crate::ebay_api_model::item_summary::ItemSummary;
use crate::ebay_finder::NotifStatus;

pub struct DiscordClient {
    inner: WebhookClient,
}

impl DiscordClient {
    pub fn new(url: &str) -> Self {
        Self {
            inner: WebhookClient::new(url),
        }
    }

    pub async fn send_item(&self, item: ItemSummary, status: NotifStatus) {
        // println!("{:?} {:?}", status, item); // TODO: put discord webhook here

        self.inner
            .send(|message| message.content("test"))
            .await
            .unwrap();
    }
}
