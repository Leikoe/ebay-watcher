use crate::{
    discord::DiscordClient,
    ebay_api_model::item_summary::{ItemSummary, ItemSummaryResponse},
    ebay_finder::NotifEvent,
};
use base64::prelude::{BASE64_STANDARD, Engine as _};
use dotenv::dotenv;
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, env, time::Duration};
use tokio::{
    signal,
    time::{Instant, sleep},
};

mod discord;
mod ebay_api_model;
mod ebay_finder;

// const QUERY: &str = "nvidia (H100,H800,A100,A800,Ampere,Hopper,L40,L40S,SXM,SXM4,48GB,40GB,HBM2,HBM3) -(RTX,Shroud,fan,cooling,blower,A2,A30,A40,16GB,P100,Laptop,HP,Lenovo,Windows,SSD,i7,i5,Pascal)";
const QUERIES: [&str; 2] = [
    "nvidia (A100,A800,A100X,A800X,H100,H800,PG530,PG520,PG,HBM3,HBM3e,PG199,DRIVE A100) -(Quadro)",
    "amd (MI250,MI250X,MI300X,MI300A,MI325X,MI350X,MI355X,MI,HBM3,HBM3e)",
];

#[derive(Deserialize, Debug, Clone)]
struct EbayTokenResp {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

struct EbayToken {
    access_token: String,
    expires_at: Instant,
}

async fn ebay_get_token(
    ebay_app_id: &str,
    ebay_app_secret: &str,
) -> Result<EbayToken, reqwest::Error> {
    println!("getting a new token!");
    let client = Client::new();
    let resp = client
        .post("https://api.ebay.com/identity/v1/oauth2/token")
        .header(
            "Authorization",
            format!(
                "Basic {}",
                BASE64_STANDARD.encode(format!("{}:{}", ebay_app_id, ebay_app_secret))
            ),
        )
        .body("grant_type=client_credentials&scope=https%3A%2F%2Fapi.ebay.com%2Foauth%2Fapi_scope")
        .send()
        .await?;
    let token: EbayTokenResp = resp.json().await?;

    Ok(EbayToken {
        access_token: token.access_token,
        expires_at: Instant::now() + Duration::from_secs(token.expires_in),
    })
}

async fn run(
    webhook_client: &DiscordClient,
    http_client: &Client,
    ebay_app_id: &str,
    ebay_app_secret: &str,
) -> Result<(), String> {
    let mut ebay_token = ebay_get_token(&ebay_app_id, &ebay_app_secret)
        .await
        .map_err(|e| e.to_string())?;

    let mut ids_db: HashMap<String, ItemSummary> = HashMap::new();
    let mut new_db = true;

    loop {
        // if token soon expired, request a new one
        if Instant::now() + Duration::from_secs(5) > ebay_token.expires_at {
            println!("[LOG] ebay token expired or expiring soon, requesting a new one");
            ebay_token = ebay_get_token(ebay_app_id, ebay_app_secret)
                .await
                .map_err(|e| e.to_string())?;
        }

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
                .header("Authorization", format!("Bearer {}", ebay_token.access_token))
                .send()
                .await.map_err(|e| e.to_string())?;

            let status = resp.status();
            if status != 200 {
                let txt = resp.text().await;
                println!("{:?}", txt);
                return Err(format!("Got Status {:?}: {:?}", status, txt));
            }
            let items: ItemSummaryResponse = resp.json().await.map_err(|e| e.to_string())?;

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
                                if let Err(e) = webhook_client
                                    .send_item(NotifEvent::UPDATED, &item, Some(old_item))
                                    .await
                                {
                                    eprintln!("coudln't send price update webhook ({})", e);
                                    continue;
                                }
                            }
                        }
                        None => {
                            // new item
                            new_items_count += 1;
                            if let Err(e) = webhook_client
                                .send_item(NotifEvent::CREATED, &item, None)
                                .await
                            {
                                eprintln!("coudln't send new item webhook ({})", e);
                                continue;
                            }
                        }
                    };
                }
                ids_db.insert(id.to_owned(), item.clone()); // TODO: do the flip flop technique to never OOM
            }
        }

        println!(
            "Found {} new items and {} updated prices",
            new_items_count, updated_items_count
        );
        new_db = false; // db is inited after first loop
        sleep(Duration::from_secs(60)).await;
    }
}

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
    let ebay_app_id = env::vars()
        .find(|(k, _)| k == "EBAY_APP_ID")
        .map(|(_, t)| t)
        .expect("couldn't find the EBAY_APP_ID env var");
    let ebay_app_secret = env::vars()
        .find(|(k, _)| k == "EBAY_APP_SECRET")
        .map(|(_, t)| t)
        .expect("couldn't find the EBAY_APP_SECRET env var");
    let webhook_url = env::vars()
        .find(|(k, _)| k == "DISCORD_WEBHOOK_URL")
        .map(|(_, t)| t)
        .expect("couldn't find the WEBHOOK_URL env var");

    // create clients
    let webhook_client = DiscordClient::new(&webhook_url);
    let http_client = Client::new();

    webhook_client
        .send_message(&format!(
            "**Starting Up!**\nTracking: \n{}",
            QUERIES.map(|q| format!("- {}", q)).join("\n")
        ))
        .await
        .expect("couldn't send startup message");

    tokio::select! {
        e = run(&webhook_client, &http_client, &ebay_app_id, &ebay_app_secret) => {
            eprintln!("stopping bot, reason: {:?}", e);
            webhook_client.send_message("bot died, check console for reason").await.expect("couldn't send message")
        }
        _ = signal::ctrl_c() => {
            webhook_client.send_message("stopping bot").await.expect("couldn't send message")
        }
    };
}
