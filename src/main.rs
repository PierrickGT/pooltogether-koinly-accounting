use pooltogether_koinly_accounting::{
    bot::Bot,
    config::Config,
    constants::get_liquidation_router_address,
    init::{print_banner, setup_logger},
    log_info_cyan,
};

use alloy::{
    network::AnyNetwork,
    providers::{Provider, ProviderBuilder},
    rpc::types::{BlockNumberOrTag, Filter},
};
use chrono::DateTime;
use colored::Colorize;
use eyre::Result;
use reqwest::redirect::Policy;
use indicatif::{MultiProgress, ProgressBar};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

#[derive(Debug, Deserialize)]
struct BlockscoutBlockResponse {
    #[serde(rename = "blockNumber")]
    block_number: String,
}

#[derive(Debug, Deserialize)]
struct BlockscoutResponse<T> {
    result: T,
}

async fn get_block_by_timestamp(client: &reqwest::Client, api_key: &str, timestamp: u64) -> Result<u64> {
    let url = format!(
        "https://optimism.blockscout.com/api?module=block&action=getblocknobytime&timestamp={}&closest=before&apikey={}",
        timestamp, api_key
    );
    let resp: BlockscoutResponse<BlockscoutBlockResponse> = client.get(&url).send().await?.json().await?;
    Ok(resp.result.block_number.parse()?)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<()> {
    let config = Config::read_from_dotenv().await?;
    let blockscout_api_key = std::env::var("BLOCKSCOUT_API_KEY")?;
    let http_client = reqwest::Client::builder().redirect(Policy::limited(10)).build()?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .with_recommended_fillers()
            .network::<AnyNetwork>()
            .on_http(config.http_rpc.clone()),
    );

    let bot = Bot::new(provider.clone(), config.clone());

    let from_block = get_block_by_timestamp(&http_client, &blockscout_api_key, config.start_timestamp).await?;
    let to_block = get_block_by_timestamp(&http_client, &blockscout_api_key, config.end_timestamp).await?;

    let multi_progress = MultiProgress::new();

    setup_logger(multi_progress.clone())?;
    print_banner();

    let increment = 2000;
    let progress_bar =
        multi_progress.add(ProgressBar::new(((to_block - from_block) / increment) + 2));

    let date = DateTime::from_timestamp(config.start_timestamp as i64, 0).unwrap();
    let filename = date.format("./results/%Y-%m.csv").to_string();

    log_info_cyan!("Creating or overwriting CSV file: {}", filename);

    // Opens the file in write mode and creates it if it doesn't exist.
    // If the file already exists, it will be overwritten.
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(filename)?;

    let mut wtr = csv::Writer::from_writer(file);

    // Loop through the blocks in the range by increment.
    for from_block_number in (from_block..=to_block).step_by(increment as usize) {
        let to_block_number = std::cmp::min(from_block_number + increment, to_block);

        // Split into subranges of max 500 blocks due to RPC limits.
        let mut sub_from = from_block_number;
        while sub_from <= to_block_number {
            let sub_to = std::cmp::min(sub_from + 499, to_block_number);

            // Create a filter to watch for liquidations.
            let filter = Filter::new()
                .address(get_liquidation_router_address(config.chain_id))
                .event("SwappedExactAmountOut(address,address,address,uint256,uint256,uint256,uint256)")
                .from_block(BlockNumberOrTag::Number(sub_from))
                .to_block(BlockNumberOrTag::Number(sub_to));

            // Get all logs from the latest block that match the filter.
            let logs = Retry::spawn(
                FixedInterval::new(Duration::from_secs(2)).take(3),
                || async {
                    let attempt = provider.get_logs(&filter).await;
                    match attempt {
                        Ok(logs) => Ok(logs),
                        Err(e) => {
                            println!("Error getting logs: {:?}, retrying...", e);
                            Err(e)
                        }
                    }
                },
            )
            .await
            .unwrap();

            for log in logs {
                if let Some(koinly_data) = bot.decode_liquidation_router_event(log).await {
                    bot.write_to_koinly_csv(&mut wtr, koinly_data).await;
                }
            }

            sub_from = sub_to + 1;
        }

        // Increment the progress bar after each iteration.
        progress_bar.inc(1);
    }

    wtr.flush()?;

    log_info_cyan!("Transactions processed!");

    progress_bar.finish();
    multi_progress.remove(&progress_bar);

    Ok(())
}
