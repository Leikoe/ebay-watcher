use serenity::all::{CreateEmbed, CreateEmbedFooter};
use serenity::builder::ExecuteWebhook;
use serenity::http::Http;
use serenity::model::webhook::Webhook;

use crate::ebay_api_model::item_summary::ItemSummary;
use crate::ebay_finder::NotifEvent;

pub struct DiscordClient {
    http_client: Http,
    url: String,
}

impl DiscordClient {
    pub fn new(url: &str) -> Self {
        Self {
            http_client: Http::new(""),
            url: url.to_owned(),
        }
    }

    pub async fn send_item(
        &self,
        listing: &ItemSummary,
        event: NotifEvent,
    ) -> Result<(), serenity::Error> {
        let webhook = Webhook::from_url(&self.http_client, &self.url)
            .await
            .expect("couldn't validate webhook url"); // TODO: this might fire a request to discord's api ...

        println!("[DISCORD] Sending {:?} {:?}", event, listing);

        let builder = ExecuteWebhook::new().username("Watcher").embed({
            let mut emb = CreateEmbed::new()
                .title(&listing.title)
                .url(&format!(
                    "https://www.ebay.com/itm/{}",
                    listing.id().expect("couldn't parse id")
                ))
                .footer(CreateEmbedFooter::new("Ebay Watcher").icon_url(
                    "https://i.pinimg.com/564x/d8/c1/58/d8c15881c29b6ccd441cefeecbf8d7bc.jpg",
                ))
                .image(&listing.image.image_url)
                .color(if listing.is_auction() {
                    (0xE7, 0x6F, 0x51)
                } else {
                    (0x26, 0x46, 0x53)
                });
            if listing.is_auction() {
                if let Some((price, currency)) = listing.current_bid_price() {
                    emb = emb.field("Current Price", &format!("{} {}", price, currency), false);
                }
                if let Some(t) = listing.end_timestamp() {
                    emb = emb.field("Ends in", &format!("<t:{}:R>", t), false)
                }
            }
            if let Some((price, currency)) = listing.bin_price() {
                emb = emb.field("BIN Price", &format!("{} {}", price, currency), false);
            }
            emb.field("Condition", &listing.condition, false).field(
                "Listing Type",
                &listing.buying_options.join(", "),
                false,
            )
        });
        webhook
            .execute(&self.http_client, false, builder)
            .await
            .map(|_| ())
    }
}
