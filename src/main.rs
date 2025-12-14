use crate::{discord::DiscordClient, ebay_finder::NotifEvent};
use dotenv::dotenv;
use ebay_api_model::item_summary::{ItemSummary, ItemSummaryResponse};
use reqwest::{
    ClientBuilder,
    header::{HeaderMap, HeaderValue},
};
use std::env;

mod discord;
mod ebay_api_model;
mod ebay_finder;

// const QUERY: &str = "nvidia (H100,H800,A100,A800,Ampere,Hopper,L40,L40S,SXM,SXM4,48GB,40GB,HBM2,HBM3) -(RTX,Shroud,fan,cooling,blower,A2,A30,A40,16GB,P100,Laptop,HP,Lenovo,Windows,SSD,i7,i5,Pascal)";
const QUERY: &str = "nvidia PG530";

#[tokio::main]
async fn main() {
    if QUERY.len() > 100 {
        println!(
            "QUERY longer than max, will be truncated to '{}'",
            QUERY.chars().take(100).collect::<String>()
        );
    }

    let _ = dotenv().ok();

    // get env vars
    let token = env::vars()
        .find(|(k, _)| k == "TOKEN")
        .map(|(_, t)| t)
        .expect("couldn't find the TOKEN env var");
    let webhook_url = env::vars()
        .find(|(k, _)| k == "WEBHOOK_URL")
        .map(|(_, t)| t)
        .expect("couldn't find the WEBHOOK_URL env var");

    // create clients
    let webhook_client = DiscordClient::new(&webhook_url);
    let http_client = ClientBuilder::default()
        .default_headers({
            let mut default_headers = HeaderMap::new();
            default_headers.insert(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", token)).expect("TOKEN should be ascii"),
            );
            default_headers
        })
        .build()
        .expect("couldn't build web client");

    let resp = http_client
        .get(format!(
            "https://api.ebay.com/buy/browse/v1/item_summary/search?q={}&limit=5",
            QUERY
        ))
        .header("X-EBAY-C-MARKETPLACE-ID", "EBAY-US")
        .send()
        .await
        .unwrap();

    println!("{:?}", resp.headers());

    if resp.status() != 200 {
        println!("{}", resp.text().await.unwrap());
        return;
    }
    let items: ItemSummaryResponse = resp.json().await.unwrap();

    println!("Items:");
    for item in items.item_summaries {
        webhook_client
            .send_item(&item, NotifEvent::CREATED)
            .await
            .expect("couldn't send webhook");
    }
}
