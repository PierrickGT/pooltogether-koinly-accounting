use crate::{
    config::{Config, GoogleDriveConfig},
    constants::{
        get_asset_decimals, get_asset_symbol, get_underlying_asset_address, POOL_OPTIMISM_ADDRESS,
    },
    log_info_cyan,
};

use alloy::{
    network::{primitives::BlockTransactionsKind, AnyNetwork},
    primitives::{utils::format_units, Address, U64},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
        Identity, Provider, RootProvider,
    },
    rpc::types::{AnyTransactionReceipt, Log},
    sol,
    transports::{
        http::{Client, Http},
        Transport,
    },
};
use chrono::{DateTime, Utc};
use colored::Colorize;
use google_drive::{types::File, Client as GoogleDriveClient};
use op_alloy_rpc_types::OptimismTransactionReceiptFields;
use sheets::{
    spreadsheets::Spreadsheets,
    types::{
        DateTimeRenderOption, Dimension, InsertDataOption, ValueInputOption, ValueRange,
        ValueRenderOption,
    },
    Client as GoogleSheetsClient,
};
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone)]
pub struct KoinlyData {
    pub date: DateTime<Utc>,
    pub amount_in: String,
    pub amount_in_symbol: String,
    pub amount_out: String,
    pub amount_out_symbol: String,
    pub fee: String,
    pub fee_symbol: String,
    pub tx_hash: String,
}

sol! {
    event SwappedExactAmountOut(address indexed liquidationPair, address indexed sender, address indexed receiver, uint256 amountOut, uint256 amountInMax, uint256 amountIn, uint256 deadline);
}

type Filler = FillProvider<
    JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
    RootProvider<Http<Client>, AnyNetwork>,
    Http<Client>,
    AnyNetwork,
>;

#[derive(Clone)]
pub struct Bot<F, T, P> {
    /// Provider
    provider: Arc<P>,
    /// Chain ID
    chain_id: U64,
    /// Address of the sender for which to record liquidation transactions
    sender: Address,
    /// Google Drive config
    google_drive_config: GoogleDriveConfig,
    phantom: PhantomData<(F, T)>,
}

impl<T, P> Bot<Filler, T, P>
where
    T: Transport + Clone,
    P: Provider<T, AnyNetwork> + Clone,
{
    pub fn new(provider: Arc<P>, config: Config) -> Self {
        Self {
            provider: provider.clone(),
            chain_id: config.chain_id,
            sender: config.sender,
            google_drive_config: config.google_drive,
            phantom: PhantomData,
        }
    }

    /// Decode liquidation router SwappedExactAmountOut event
    pub async fn decode_liquidation_router_event(&self, log: Log) -> Option<KoinlyData> {
        let tx_hash = log.transaction_hash.unwrap();

        let block = self
            .provider
            .get_block(
                log.block_number.unwrap().into(),
                BlockTransactionsKind::Full,
            )
            .await
            .unwrap()
            .unwrap();

        let date_utc = DateTime::from_timestamp(block.header.timestamp as i64, 0).unwrap();

        let receipt = AnyTransactionReceipt::from(
            self.provider
                .get_transaction_receipt(tx_hash)
                .await
                .unwrap()
                .unwrap(),
        );

        let optimism_fields: OptimismTransactionReceiptFields =
            receipt.other.clone().deserialize_into().unwrap();

        let mut event_data: Option<KoinlyData> = None;

        if let Ok(decoded_log) = log.log_decode::<SwappedExactAmountOut>() {
            let event = decoded_log.inner.data;

            // Return early if the transaction is not from the sender.
            if event.sender != self.sender {
                return None;
            }

            let amount_out_asset_address =
                get_underlying_asset_address(self.chain_id, event.liquidationPair);

            event_data = Some(KoinlyData {
                date: date_utc,
                amount_in: format_units(
                    event.amountIn,
                    get_asset_decimals(self.chain_id, *POOL_OPTIMISM_ADDRESS),
                )
                .unwrap(),
                amount_in_symbol: get_asset_symbol(self.chain_id, *POOL_OPTIMISM_ADDRESS)
                    .to_string(),
                amount_out: format_units(
                    event.amountOut,
                    get_asset_decimals(self.chain_id, amount_out_asset_address),
                )
                .unwrap(),
                amount_out_symbol: get_asset_symbol(self.chain_id, amount_out_asset_address)
                    .to_string(),
                fee: format_units(
                    receipt.gas_used * receipt.effective_gas_price
                        + optimism_fields.l1_block_info.l1_fee.unwrap(),
                    18,
                )
                .unwrap(),
                fee_symbol: "ETH".to_string(),
                tx_hash: format!("0x{:064x}", tx_hash),
            });
        }

        event_data
    }

    /// Write liquidation data in the Koinly CSV fromat to a Google Sheets spreadsheet
    pub async fn write_to_koinly_csv(&self, data: KoinlyData) {
        let google_drive = GoogleDriveClient::new(
            self.google_drive_config.client_id.clone(),
            self.google_drive_config.client_secret.clone(),
            self.google_drive_config.redirect_uri.clone(),
            self.google_drive_config.token.clone(),
            self.google_drive_config.refresh_token.clone(),
        );

        let google_sheets = GoogleSheetsClient::new(
            self.google_drive_config.client_id.clone(),
            self.google_drive_config.client_secret.clone(),
            self.google_drive_config.redirect_uri.clone(),
            self.google_drive_config.token.clone(),
            self.google_drive_config.refresh_token.clone(),
        );

        let _ = google_drive.refresh_access_token().await;
        let _ = google_sheets.refresh_access_token().await;

        let spreadsheets = Spreadsheets::new(google_sheets);

        let mime_type = "application/vnd.google-apps.spreadsheet".to_string();
        let parent_folder_id = self.google_drive_config.folder_id.clone();

        let query = format!(
            "mimeType = '{}' and '{}' in parents",
            mime_type, self.google_drive_config.folder_id
        );

        let files = google_drive
            .files()
            .list(
                "user", "", false, "", false, "", 0, "", &query, "", false, false, "",
            )
            .await
            .unwrap()
            .body;

        let filename = data.date.format("%Y-%m").to_string();
        let range = String::from("Sheet1!A1:H1");

        for file in files {
            // If file exists, write Koinly data to it.
            if file.name == filename {
                let _ = spreadsheets
                    .values_append(
                        &file.id.to_string(),
                        &range.to_string(),
                        false,
                        InsertDataOption::InsertRows,
                        DateTimeRenderOption::SerialNumber,
                        ValueRenderOption::FormattedValue,
                        ValueInputOption::Raw,
                        &ValueRange {
                            major_dimension: Some(Dimension::Rows),
                            range,
                            values: vec![vec![
                                data.date.to_string(),
                                data.amount_in,
                                data.amount_in_symbol,
                                data.amount_out,
                                data.amount_out_symbol,
                                data.fee,
                                data.fee_symbol,
                                data.tx_hash,
                            ]],
                        },
                    )
                    .await;

                log_info_cyan!("Updated spreadsheet {} with liquidation data!", filename);

                return;
            }
        }

        // If file doesn't exist, create it and write Koinly data to it.
        let new_file = File {
            created_time: Some(Utc::now()),
            name: filename,
            mime_type,
            parents: vec![parent_folder_id],
            ..Default::default()
        };

        let created_file = google_drive
            .files()
            .create(false, "", false, "", false, false, false, &new_file)
            .await
            .unwrap();

        log_info_cyan!("Create new spreadsheet: {:?}", created_file.body.name);

        let new_spreadsheet_id = &created_file.body.id.to_string();

        let _ = spreadsheets
            .values_append(
                new_spreadsheet_id,
                &range.to_string(),
                false,
                InsertDataOption::InsertRows,
                DateTimeRenderOption::SerialNumber,
                ValueRenderOption::FormattedValue,
                ValueInputOption::Raw,
                &ValueRange {
                    major_dimension: Some(Dimension::Rows),
                    range: range.clone(),
                    values: vec![vec![
                        String::from("Date"),
                        String::from("Sent Amount"),
                        String::from("Sent Currency"),
                        String::from("Received Amount"),
                        String::from("Received Currency"),
                        String::from("Fee Amount"),
                        String::from("Fee Currency"),
                        String::from("TxHash"),
                    ]],
                },
            )
            .await;

        let _ = spreadsheets
            .values_append(
                new_spreadsheet_id,
                &range.to_string(),
                false,
                InsertDataOption::InsertRows,
                DateTimeRenderOption::SerialNumber,
                ValueRenderOption::FormattedValue,
                ValueInputOption::Raw,
                &ValueRange {
                    major_dimension: Some(Dimension::Rows),
                    range,
                    values: vec![vec![
                        data.date.to_string(),
                        data.amount_in,
                        data.amount_in_symbol,
                        data.amount_out,
                        data.amount_out_symbol,
                        data.fee,
                        data.fee_symbol,
                        data.tx_hash,
                    ]],
                },
            )
            .await;

        log_info_cyan!("Inserted liquidation data into new spreadsheet!");
    }
}
