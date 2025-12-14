use webhook::client::{WebhookClient, WebhookResult};

use crate::ebay_api_model::item_summary::ItemSummary;
use crate::ebay_finder::NotifEvent;

pub struct DiscordClient {
    inner: WebhookClient,
}

impl DiscordClient {
    pub fn new(url: &str) -> Self {
        Self {
            inner: WebhookClient::new(url),
        }
    }

    pub async fn send_item(&self, listing: &ItemSummary, event: NotifEvent) -> WebhookResult<()> {
        println!("[DISCORD] Sending {:?} {:?}", event, listing);

        self.inner
            .send(|message| {
                message
                    .username("Thoo")
                    .embed(|e| {
                    let mut embed = e.author(&format!("{:?}", event), None, None)
                        .title(&listing.title)
                        .url(&format!("https://www.ebay.com/itm/{}", listing.item_id))
                        .field(
                            "Price",
                            &format!("{} {}", listing.price(), listing.price.currency),
                            false,
                        )
                        .field("Condition", &listing.condition, false)
                        .field("Listing Type", &listing.buying_options.join(", "), false)
                        .footer("Ebay Watcher", Some("https://i.pinimg.com/564x/d8/c1/58/d8c15881c29b6ccd441cefeecbf8d7bc.jpg".to_owned()))
                        .image(&listing.image.image_url)
                        .color(if listing.is_auction() { "0xE76F51" } else { "0x264653"});
                    // if listing.is_auction() {
                    //     if let Some(t) = listing.end_timestamp() {
                    //         embed = embed.field("Ends in", &format!("<t:{}:R>", t), false)
                    //     }
                    // }
                    embed
                })
            })
            .await
            .map(|_| ()) // discord the bool from Ok variant
    }
}
