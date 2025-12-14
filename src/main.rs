use crate::{
    discord::DiscordClient,
    ebay_api_model::item_summary::{ItemSummary, ItemSummaryResponse},
    ebay_finder::NotifEvent,
};
use dotenv::dotenv;
use reqwest::{
    ClientBuilder,
    header::{HeaderMap, HeaderValue},
};
use std::{collections::HashMap, env, time::Duration};
use tokio::time::sleep;

mod discord;
mod ebay_api_model;
mod ebay_finder;

// const QUERY: &str = "nvidia (H100,H800,A100,A800,Ampere,Hopper,L40,L40S,SXM,SXM4,48GB,40GB,HBM2,HBM3) -(RTX,Shroud,fan,cooling,blower,A2,A30,A40,16GB,P100,Laptop,HP,Lenovo,Windows,SSD,i7,i5,Pascal)";
const QUERIES: [&str; 2] = [
    "nvidia (A100,A800,A100X,A800X,H100,H800,PG530,PG520,PG,HBM3,HBM3e,PG199,DRIVE A100)",
    "amd (MI250,MI250X,MI300X,MI300A,MI325X,MI350X,MI355X,MI,HBM3,HBM3e)",
];

#[tokio::main]
async fn main() {
    for query in QUERIES {
        if query.len() > 100 {
            println!(
                "query longer than max, will be truncated to '{}'",
                query.chars().take(100).collect::<String>()
            );
        }
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

    let mut ids_db: HashMap<String, ItemSummary> = HashMap::new();
    let mut new_db = true;

    webhook_client
        .send_message("Starting Up!")
        .await
        .expect("couldn't send startup message");

    loop {
        let mut new_items_count: usize = 0;
        let mut updated_items_count: usize = 0;
        println!("requesting items from ebay..");
        for query in QUERIES {
            println!("\tq={}", query);
            let resp = http_client
                .get(format!(
                    "https://api.ebay.com/buy/browse/v1/item_summary/search?q={}&limit=200&sort=newlyListed",
                    query
                ))
                .header("X-EBAY-C-MARKETPLACE-ID", "EBAY-US")
                .send()
                .await
                .unwrap();

            if resp.status() != 200 {
                println!("{}", resp.text().await.unwrap());
                return;
            }
            let items: ItemSummaryResponse = resp.json().await.unwrap();

            for item in &items.item_summaries {
                let Some(id) = item.id() else {
                    eprintln!("coudln't get item id from ({})", item.item_id);
                    continue;
                };

                // if new item and not initializing db
                if !new_db {
                    match ids_db.get(id) {
                        Some(old_item) => {
                            if item.price != old_item.price {
                                updated_items_count += 1;
                                webhook_client
                                    .send_item(NotifEvent::UPDATED, &item, Some(old_item))
                                    .await
                                    .expect("couldn't send webhook");
                            }
                        }
                        None => {
                            // new item
                            new_items_count += 1;
                            webhook_client
                                .send_item(NotifEvent::CREATED, &item, None)
                                .await
                                .expect("couldn't send webhook");
                        }
                    };
                }
                ids_db.insert(id.to_owned(), item.clone()); // TODO: do the flip flop technique to never OOM
            }
        }

        println!(
            "found {} new items and {} updated prices",
            new_items_count, updated_items_count
        );
        new_db = false; // db is inited after first loop
        sleep(Duration::from_secs(60)).await;
    }
}
