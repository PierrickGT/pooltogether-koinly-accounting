use alloy::primitives::{Address, U64};
use dotenv::dotenv;
use eyre::{eyre, Result};
use reqwest::Url;
use serde::Deserialize;
use serde_json;
use std::env;
use std::fs;

#[derive(Deserialize)]
struct AccessToken {
    access_token: String,
    refresh_token: String,
}

#[derive(Deserialize)]
struct ClientSecrets {
    client_id: String,
    client_secret: String,
    redirect_uris: Vec<String>,
}

#[derive(Deserialize)]
struct Credentials {
    access_token: AccessToken,
    client_secrets: ClientSecrets,
}

#[derive(Clone)]
pub struct GoogleDriveConfig {
    pub client_id: String,
    pub client_secret: String,
    pub folder_id: String,
    pub redirect_uri: String,
    pub token: String,
    pub refresh_token: String,
}

#[derive(Clone)]
pub struct Config {
    pub http_rpc: Url,
    pub chain_id: U64,
    pub sender: Address,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub google_drive: GoogleDriveConfig,
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

        let google_drive_credentials_path: String = get_env("GOOGLE_DRIVE_CREDENTIALS_PATH")?
            .parse()
            .map_err(|_| eyre!("Failed to parse \"GOOGLE_DRIVE_CREDENTIALS_PATH\""))?;

        let google_drive_folder_id: String = get_env("GOOGLE_DRIVE_FOLDER_ID")?
            .parse()
            .map_err(|_| eyre!("Failed to parse \"GOOGLE_DRIVE_FOLDER_ID\""))?;

        let credentials_file = fs::read_to_string(google_drive_credentials_path)
            .expect("Failed to read Google Drive credentials file");

        let credentials: Credentials = serde_json::from_str(&credentials_file)
            .expect("Google Drive credentials JSON file is not well-formatted");

        let google_drive = GoogleDriveConfig {
            client_id: credentials.client_secrets.client_id,
            client_secret: credentials.client_secrets.client_secret,
            folder_id: google_drive_folder_id,
            redirect_uri: credentials.client_secrets.redirect_uris[0].clone(),
            token: credentials.access_token.access_token,
            refresh_token: credentials.access_token.refresh_token,
        };

        Ok(Self {
            http_rpc,
            chain_id,
            sender,
            start_timestamp,
            end_timestamp,
            google_drive,
        })
    }
}
