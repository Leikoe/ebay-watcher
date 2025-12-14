use serenity::all::{CreateEmbed, CreateEmbedFooter};
use serenity::builder::ExecuteWebhook;
use serenity::http::Http;
use serenity::model::webhook::Webhook;

use crate::ebay_api_model::item_summary::ItemSummary;
use crate::ebay_finder::NotifEvent;

const AVATAR_URL: &str = "https://i.pinimg.com/564x/d8/c1/58/d8c15881c29b6ccd441cefeecbf8d7bc.jpg";

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

    pub async fn send_message(&self, message: &str) -> Result<(), serenity::Error> {
        println!("[DISCORD] Seding message {:?}", message);
        let webhook = Webhook::from_url(&self.http_client, &self.url).await?; // TODO: this might fire a request to discord's api ...

        let builder = ExecuteWebhook::new()
            .avatar_url(AVATAR_URL)
            .username("Watcher")
            .content(message);

        webhook
            .execute(&self.http_client, false, builder)
            .await
            .map(|_| ())
    }

    pub async fn send_item(
        &self,
        event: NotifEvent,
        listing: &ItemSummary,
        old_listing: Option<&ItemSummary>,
    ) -> Result<(), serenity::Error> {
        println!("[DISCORD] Sending {:?} {:?}", event, listing);
        let webhook = Webhook::from_url(&self.http_client, &self.url).await?; // TODO: this might fire a request to discord's api ...

        let builder = ExecuteWebhook::new()
            .avatar_url(AVATAR_URL)
            .username("Watcher")
            .embed({
                let mut emb = CreateEmbed::new()
                    .title(&listing.title)
                    .footer(CreateEmbedFooter::new("Ebay Watcher").icon_url(AVATAR_URL))
                    .image(&listing.image.image_url)
                    .color(if listing.is_auction() {
                        (0xE7, 0x6F, 0x51)
                    } else {
                        (0x26, 0x46, 0x53)
                    });

                if let Some(id) = listing.id() {
                    emb = emb.url(&format!("https://www.ebay.com/itm/{}", id));
                }
                if listing.is_auction() {
                    let auction_price_display = if let Some(old_listing) = old_listing
                        && old_listing.current_bid_price() != listing.current_bid_price()
                    {
                        if let (Some((old_price, old_currency)), Some((price, currency))) =
                            (old_listing.current_bid_price(), listing.current_bid_price())
                        {
                            Some(format!(
                                "{} {} -> {} {}",
                                old_price, old_currency, price, currency
                            ))
                        } else {
                            None
                        }
                    } else {
                        if let Some((price, currency)) = listing.current_bid_price() {
                            Some(format!("{} {}", price, currency))
                        } else {
                            None
                        }
                    };
                    emb = emb.field(
                        "Current Price",
                        auction_price_display
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or("couldn't get bid price info :("),
                        false,
                    );
                    if let Some(t) = listing.end_timestamp() {
                        emb = emb.field("Ends in", &format!("<t:{}:R>", t), false)
                    }
                }
                if let Some(old_listing) = old_listing
                    && old_listing.bin_price() != listing.bin_price()
                {
                    if let (Some((old_price, old_currency)), Some((price, currency))) =
                        (old_listing.bin_price(), listing.bin_price())
                    {
                        emb = emb.field(
                            "BIN Price",
                            &format!("{} {} -> {} {}", old_price, old_currency, price, currency),
                            false,
                        );
                    }
                } else {
                    if let Some((price, currency)) = listing.bin_price() {
                        emb = emb.field("BIN Price", &format!("{} {}", price, currency), false);
                    }
                }
                emb.field(
                    "Condition",
                    listing
                        .condition
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or("Unknown"),
                    false,
                )
                .field("Listing Type", &listing.buying_options.join(", "), false)
            });
        webhook
            .execute(&self.http_client, false, builder)
            .await
            .map(|_| ())
    }
}
