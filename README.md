# PoolTogether Koinly Accounting

This repository contains a Rust program that queries a range of blocks and filters PoolTogether's yield liquidations for a given EVM network and sender address. It then writes the necessary data to a CSV file and formats it in the Koinly CSV format.

It currently only supports the Optimism Canary deployment, but can be easily extended to support the latest version and networks.
The script was only tested on the Optimism Canary deployment, but should work with the latest version as well.

## Demo

![Demo](./media/demo.gif)

## Usage

### Environment Variables

The following environment variables are required:

- `HTTP_RPC`: the RPC endpoint of the network to query. Should be an Archive node to query historical data.
- `CHAIN_ID`: the chain ID of the network to query in hexadecimal format (e.g., `0x0A` for Optimism).
- `SENDER_ADDRESS`: the address of the sender for which to record liquidation transactions.
- `START_TIMESTAMP`: the start timestamp in seconds at which to start querying blocks.
- `END_TIMESTAMP`: the end timestamp in seconds at which to stop querying blocks.
- `ETHERSCAN_API_KEY`: the Etherscan API key for the network to query. Used for fetching the block number at a given timestamp. For Optimism, you need to retrieve the API key at the following URL: [https://optimistic.etherscan.io/myapikey](https://optimistic.etherscan.io/myapikey)

### Running

To run the script, execute the following command:

```bash
cargo run
```
