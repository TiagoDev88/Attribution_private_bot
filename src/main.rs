use dotenv::dotenv;
use log::info;
use pretty_env_logger;
use reqwest;
use serde::Deserialize;
use std::env;
use teloxide::{prelude::*, requests::ResponseResult, types::Me};

#[derive(Deserialize)]
struct ChainStats {
    tx_count: Option<i64>, //for Numbers and Option<String> for String.
}

#[derive(Deserialize)]
struct BitcoinAddressInfo {
    chain_stats: ChainStats,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    info!("Starting bot...");

    let bot = Bot::from_env();
    teloxide::repl(bot, |message: UpdateWithCx<Bot, Message>| async move {
        handle_message(message).await;
        ResponseResult::<()>::Ok(())
    })
    .await;
}

async fn handle_message(cx: UpdateWithCx<Bot, Message>) {
    if let Some(text) = cx.update.text() {
        let user_id = cx.update.from().unwrap().id;
        println!("user id {:?}", user_id);

        let allowed_users: Vec<i64> = env::var("ALLOWED_USERS")
            .expect("ALLOWED_USERS is not defined in the .env file")
            .split(',')
            .map(|s| s.parse().expect("Failed to parse the number"))
            .collect();

        if allowed_users.contains(&user_id) {
            match check_bitcoin_address_info(text).await {
                Ok(response) => {
                    cx.answer(response).send().await.log_on_error().await;
                }
                Err(_) => {
                    cx.answer("Error verifying address.")
                        .send()
                        .await
                        .log_on_error()
                        .await;
                }
            }
        } else {
            cx.answer("You do not have permission to use this bot.")
                .send()
                .await
                .log_on_error()
                .await;
        }
    }
    println!("Some text {:?}", Some(()));
}

async fn check_bitcoin_address_info(
    address: &str,
) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();
    let api_base_url =
        env::var("API_BASE_URL").expect("API_BASE_URL is not defined in the .env file");

    let api_url = format!("{}/{}", api_base_url, address);
    println!("api url {:?}", api_url);

    let resp = reqwest::get(&api_url)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
    println!("resp {:?}", resp);
    if resp.status().is_success() {
        let api_response = resp
            .json::<BitcoinAddressInfo>()
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

        if let Some(name) = api_response.chain_stats.tx_count {
            Ok(format!("This address is attributed to: {}", name))
        } else {
            Ok("This address is not attributed.".to_string())
        }
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Address not found or API error.",
        )) as Box<dyn std::error::Error + Send>)
    }
}
