# PoolTogether Koinly Accounting

This repository contains a Rust program that queries a range of blocks and filters PoolTogether's yield liquidations for a given EVM network and sender address. It then writes the necessary data to a Google Sheets spreadsheet and formats it in the Koinly CSV format.

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
- `GOOGLE_DRIVE_CREDENTIALS_PATH`: the path to the Google Drive credentials JSON file (e.g., `./credentials/google-drive-credentials.json`).
- `GOOGLE_DRIVE_FOLDER_ID`: the ID of the Google Drive folder where the spreadsheet will be created or updated.

### Google Drive Credentials

You first need to create a Google Drive API project and retrieve the credentials OAuth 2.0 client IDs by following this guide: [https://developers.google.com/workspace/guides/create-credentials#oauth-client-id](https://developers.google.com/workspace/guides/create-credentials#oauth-client-id).

Follow the these instructions to retrieve your access and refresh tokens:
[https://github.com/PierrickGT/third-party-api-clients/tree/fix/google-drive-fields/google/sheets#basic-example](https://github.com/PierrickGT/third-party-api-clients/tree/fix/google-drive-fields/google/sheets#basic-example)

Your credentials JSON file should look like this:

```json
{
  "client_secrets": {
    "client_id": "YOUR_CLIENT_ID",
    "project_id": "YOUR_PROJECT_ID",
    "auth_uri": "https://accounts.google.com/o/oauth2/auth",
    "token_uri": "https://oauth2.googleapis.com/token",
    "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
    "client_secret": "YOUR_CLIENT_SECRET",
    "redirect_uris": ["YOUR_REDIRECT_URI"]
  },
  "access_token": {
    "token_type": "Bearer",
    "access_token": "YOUR_ACCESS_TOKEN",
    "expires_in": 3599,
    "refresh_token": "YOUR_REFRESH_TOKEN",
    "refresh_token_expires_in": 0,
    "scope": "https://www.googleapis.com/auth/drive"
  }
}
```

### Running

To run the script, execute the following command:

```bash
cargo run
```
