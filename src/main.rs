use crate::{discord::DiscordClient, ebay_finder::NotifEvent, id_db::IdDatabase};
use dotenv::dotenv;
use ebay_api_model::item_summary::ItemSummaryResponse;
use reqwest::{
    ClientBuilder,
    header::{HeaderMap, HeaderValue},
};
use std::{env, path::Path, time::Duration};
use tokio::time::sleep;

mod discord;
mod ebay_api_model;
mod ebay_finder;
mod id_db;

// const QUERY: &str = "nvidia (H100,H800,A100,A800,Ampere,Hopper,L40,L40S,SXM,SXM4,48GB,40GB,HBM2,HBM3) -(RTX,Shroud,fan,cooling,blower,A2,A30,A40,16GB,P100,Laptop,HP,Lenovo,Windows,SSD,i7,i5,Pascal)";
const QUERY: &str = "nvidia (H100,H800,A100,A800,PG530,PG520,PG)";
const IDS_DB_FILE: &str = "db.txt";

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

    let ids_db_path = Path::new(IDS_DB_FILE);
    let mut new_db = false;
    let mut ids_db = IdDatabase::from_path(ids_db_path).unwrap_or({
        println!("couldn't find ids db file, creating a new empty ids db");
        new_db = true;
        IdDatabase::new()
    });

    loop {
        let mut new_items_count: usize = 0;
        println!("requesting items from ebay..");
        let resp = http_client
            .get(format!(
                "https://api.ebay.com/buy/browse/v1/item_summary/search?q={}&limit=200&sort=newlyListed",
                QUERY
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

        for item in items.item_summaries {
            let Some(id) = item.id() else {
                eprintln!("coudln't get item id from ({})", item.item_id);
                continue;
            };

            // if new item
            if !ids_db.contains(id) && !new_db {
                new_items_count += 1;
                webhook_client
                    .send_item(&item, NotifEvent::CREATED)
                    .await
                    .expect("couldn't send webhook");
            }
            ids_db.add(id); // TODO: do the flip flop technique to never OOM
        }
        println!("found {} new items", new_items_count);
        new_db = false; // db is inited after first loop
        if let Err(e) = ids_db.save_to_path(ids_db_path) {
            eprintln!("[ERR] couldn't save db to path: {}", e);
        }
        sleep(Duration::from_secs(60)).await;
    }
}
