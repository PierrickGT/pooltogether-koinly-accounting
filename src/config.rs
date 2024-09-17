use alloy::primitives::{Address, U64};
use dotenv::dotenv;
use eyre::{eyre, Result};
use reqwest::Url;
use std::env;

#[derive(Clone)]
pub struct Config {
    pub http_rpc: Url,
    pub chain_id: U64,
    pub sender: Address,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
}

impl Config {
    pub async fn read_from_dotenv() -> Result<Self> {
        dotenv().ok();

        let get_env = |var| {
            env::var(var).map_err(|_| eyre!("Required environment variable \"{}\" not set", var))
        };

        let http_rpc = get_env("HTTP_RPC")?
            .parse()
            .map_err(|_| eyre!("Failed to parse \"HTTP_RPC\""))?;

        let chain_id = get_env("CHAIN_ID")?
            .parse::<U64>()
            .map_err(|_| eyre!("Failed to parse \"CHAIN_ID\""))?;

        let sender = get_env("SENDER_ADDRESS")?
            .parse::<Address>()
            .map_err(|_| eyre!("Failed to parse \"SENDER_ADDRESS\""))?;

        let start_timestamp = get_env("START_TIMESTAMP")?
            .parse::<u64>()
            .map_err(|_| eyre!("Failed to parse \"START_TIMESTAMP\""))?;

        let end_timestamp = get_env("END_TIMESTAMP")?
            .parse::<u64>()
            .map_err(|_| eyre!("Failed to parse \"END_TIMESTAMP\""))?;

        Ok(Self {
            http_rpc,
            chain_id,
            sender,
            start_timestamp,
            end_timestamp,
        })
    }
}
