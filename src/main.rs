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
use alloy_chains::Chain;
use colored::Colorize;
use eyre::Result;
use foundry_block_explorers::Client;
use indicatif::{MultiProgress, ProgressBar};
use std::sync::Arc;

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<()> {
    let config = Config::read_from_dotenv().await?;
    let etherscan = Client::new_from_env(Chain::from_id(config.chain_id.try_into().unwrap()))?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .with_recommended_fillers()
            .network::<AnyNetwork>()
            .on_http(config.http_rpc.clone()),
    );

    let bot = Bot::new(provider.clone(), config.clone());

    let from_block = etherscan
        .get_block_by_timestamp(config.start_timestamp, "before")
        .await?
        .block_number
        .as_number()
        .unwrap()
        .try_into()
        .unwrap();

    let to_block = etherscan
        .get_block_by_timestamp(config.end_timestamp, "before")
        .await?
        .block_number
        .as_number()
        .unwrap()
        .try_into()
        .unwrap();

    let multi_progress = MultiProgress::new();

    setup_logger(multi_progress.clone())?;
    print_banner();

    let increment = 2000;
    let progress_bar =
        multi_progress.add(ProgressBar::new(((to_block - from_block) / increment) + 2));

    // Loop through the blocks in the range by increment.
    for from_block_number in (from_block..=to_block).step_by(increment as usize) {
        let to_block_number = std::cmp::min(from_block_number + increment, to_block);

        // Create a filter to watch for liquidations.
        let filter = Filter::new()
            .address(get_liquidation_router_address(config.chain_id))
            .event("SwappedExactAmountOut(address,address,address,uint256,uint256,uint256,uint256)")
            .from_block(BlockNumberOrTag::Number(from_block_number))
            .to_block(BlockNumberOrTag::Number(to_block_number));

        // Get all logs from the latest block that match the filter.
        let logs = provider.get_logs(&filter).await?;

        for log in logs {
            if let Some(koinly_data) = bot.decode_liquidation_router_event(log).await {
                bot.write_to_koinly_csv(koinly_data).await;
            }
        }

        // Increment the progress bar after each iteration.
        progress_bar.inc(1);
    }

    log_info_cyan!("Transactions processed!");

    progress_bar.finish();
    multi_progress.remove(&progress_bar);

    Ok(())
}
