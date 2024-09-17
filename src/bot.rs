use crate::{
    config::Config,
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
use chrono::DateTime;
use colored::Colorize;
use op_alloy_rpc_types::OptimismTransactionReceiptFields;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Clone, Serialize)]
pub struct KoinlyData {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Sent Amount")]
    amount_in: String,
    #[serde(rename = "Sent Currency")]
    amount_in_symbol: String,
    #[serde(rename = "Received Amount")]
    amount_out: String,
    #[serde(rename = "Received Currency")]
    amount_out_symbol: String,
    #[serde(rename = "Fee Amount")]
    fee: String,
    #[serde(rename = "Fee Currency")]
    fee_symbol: String,
    #[serde(rename = "TxHash")]
    tx_hash: String,
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
pub struct Bot<F, T, P, W> {
    /// Provider
    provider: Arc<P>,
    /// Chain ID
    chain_id: U64,
    /// Address of the sender for which to record liquidation transactions
    sender: Address,
    phantom: PhantomData<(F, T, W)>,
}

impl<T, P, W> Bot<Filler, T, P, W>
where
    T: Transport + Clone,
    P: Provider<T, AnyNetwork> + Clone,
    W: std::io::Write,
{
    pub fn new(provider: Arc<P>, config: Config) -> Self {
        Self {
            provider: provider.clone(),
            chain_id: config.chain_id,
            sender: config.sender,
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
                date: date_utc.to_string(),
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

    /// Write liquidation data into the CSV file
    pub async fn write_to_koinly_csv(&self, wtr: &mut csv::Writer<W>, data: KoinlyData) {
        let _ = wtr.serialize(KoinlyData {
            date: data.date.to_string(),
            amount_in: data.amount_in,
            amount_in_symbol: data.amount_in_symbol,
            amount_out: data.amount_out,
            amount_out_symbol: data.amount_out_symbol,
            fee: data.fee,
            fee_symbol: data.fee_symbol,
            tx_hash: data.tx_hash,
        });

        log_info_cyan!("Inserted liquidation data into CSV!");
    }
}
