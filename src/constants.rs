use alloy::primitives::{Address, U64};
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Network chain IDs
pub static MAINNET_CHAIN_ID: Lazy<U64> = Lazy::new(|| U64::from(1));
pub static ARBITRUM_CHAIN_ID: Lazy<U64> = Lazy::new(|| U64::from(42161));
pub static BASE_CHAIN_ID: Lazy<U64> = Lazy::new(|| U64::from(8453));
pub static OPTIMISM_CHAIN_ID: Lazy<U64> = Lazy::new(|| U64::from(10));

/// Underlying asset decimals
pub static DAI_DECIMALS: Lazy<u8> = Lazy::new(|| 18);
pub static USDC_DECIMALS: Lazy<u8> = Lazy::new(|| 6);
pub static USDCE_DECIMALS: Lazy<u8> = Lazy::new(|| 6);
pub static WETH_DECIMALS: Lazy<u8> = Lazy::new(|| 18);
pub static POOL_DECIMALS: Lazy<u8> = Lazy::new(|| 18);

/// Underlying asset addresses
pub static DAI_OPTIMISM_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1"
        .parse()
        .unwrap()
});

pub static POOL_OPTIMISM_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0x395Ae52bB17aef68C2888d941736A71dC6d4e125"
        .parse()
        .unwrap()
});

pub static USDC_OPTIMISM_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85"
        .parse()
        .unwrap()
});

pub static USDCE_OPTIMISM_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0x7F5c764cBc14f9669B88837ca1490cCa17c31607"
        .parse()
        .unwrap()
});

pub static WETH_OPTIMISM_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0x4200000000000000000000000000000000000006"
        .parse()
        .unwrap()
});

/// Liquidation pair contract addresses
pub static PDAI_LIQUIDATION_PAIR_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0x7169526daBFD1cDdE174a0A7d8c75DeB582d0990"
        .parse()
        .unwrap()
});

pub static PUSDC_LIQUIDATION_PAIR_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0x217ef9C355f7eb59C789e0471dc1f4398e004EDc"
        .parse()
        .unwrap()
});

pub static PUSDCE_LIQUIDATION_PAIR_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0xe7680701a2794E6E0a38aC72630c535B9720dA5b"
        .parse()
        .unwrap()
});

pub static PWETH_LIQUIDATION_PAIR_ADDRESS: Lazy<Address> = Lazy::new(|| {
    "0xde5deFa124faAA6d85E98E56b36616d249e543Ca"
        .parse()
        .unwrap()
});

/// Define a lazy-initialized map that maps network IDs to a map of liquidation pairs to their corresponding underlying asset addresses (i.e. tokenOut underlying asset address)
pub static UNDERLYING_ASSET_ADDRESSES: Lazy<HashMap<U64, HashMap<Address, Address>>> =
    Lazy::new(|| {
        let mut map = HashMap::new();

        // Add addresses for Optimism
        let mut optimism_tokens = HashMap::new();

        optimism_tokens.insert(*PDAI_LIQUIDATION_PAIR_ADDRESS, *DAI_OPTIMISM_ADDRESS);
        optimism_tokens.insert(*PUSDC_LIQUIDATION_PAIR_ADDRESS, *USDC_OPTIMISM_ADDRESS);
        optimism_tokens.insert(*PUSDCE_LIQUIDATION_PAIR_ADDRESS, *USDCE_OPTIMISM_ADDRESS);
        optimism_tokens.insert(*PWETH_LIQUIDATION_PAIR_ADDRESS, *WETH_OPTIMISM_ADDRESS);

        map.insert(*OPTIMISM_CHAIN_ID, optimism_tokens);

        map
    });

/// Define a lazy-initialized map that maps network IDs to a map of assets to their corresponding token decimals
pub static ASSET_DECIMALS: Lazy<HashMap<U64, HashMap<Address, u8>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Add addresses for Optimism
    let mut optimism_decimals = HashMap::new();

    optimism_decimals.insert(*DAI_OPTIMISM_ADDRESS, *DAI_DECIMALS);
    optimism_decimals.insert(*USDC_OPTIMISM_ADDRESS, *USDC_DECIMALS);
    optimism_decimals.insert(*USDCE_OPTIMISM_ADDRESS, *USDCE_DECIMALS);
    optimism_decimals.insert(*WETH_OPTIMISM_ADDRESS, *WETH_DECIMALS);
    optimism_decimals.insert(*POOL_OPTIMISM_ADDRESS, *POOL_DECIMALS);

    map.insert(*OPTIMISM_CHAIN_ID, optimism_decimals);

    map
});

/// Define a lazy-initialized map that maps network IDs to a map of assets to their corresponding token symbols
pub static ASSET_SYMBOLS: Lazy<HashMap<U64, HashMap<Address, &str>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Add addresses for Optimism
    let mut optimism_symbols = HashMap::new();

    optimism_symbols.insert(*DAI_OPTIMISM_ADDRESS, "DAI");
    optimism_symbols.insert(*USDC_OPTIMISM_ADDRESS, "USDC");
    optimism_symbols.insert(*USDCE_OPTIMISM_ADDRESS, "USDC.E");
    optimism_symbols.insert(*WETH_OPTIMISM_ADDRESS, "WETH");
    optimism_symbols.insert(*POOL_OPTIMISM_ADDRESS, "POOL");

    map.insert(*OPTIMISM_CHAIN_ID, optimism_symbols);

    map
});

/// Define a lazy-initialized map that maps network IDs to liquidation router contract addresses
pub static LIQUIDATION_ROUTER_ADDRESSES: Lazy<HashMap<U64, Address>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        *OPTIMISM_CHAIN_ID,
        "0xB9Fba7B2216167DCdd1A7AE0a564dD43E1b68b95"
            .parse()
            .unwrap(),
    );
    map
});

/// Get liquidation router address for a given chain id
pub fn get_liquidation_router_address(chain_id: U64) -> Address {
    *LIQUIDATION_ROUTER_ADDRESSES
        .get(&chain_id)
        .unwrap_or_else(|| {
            panic!(
                "No liquidation router address found for the given chain ID: {}",
                chain_id,
            )
        })
}

/// Get the underlying asset address for a given liquidation pair on a specific network
pub fn get_underlying_asset_address(chain_id: U64, liquidation_pair: Address) -> Address {
    *UNDERLYING_ASSET_ADDRESSES
        .get(&chain_id)
        .unwrap_or_else(|| {
            panic!(
                "No underlying asset addresses found for the given chain ID: {}",
                chain_id,
            )
        })
        .get(&liquidation_pair)
        .unwrap_or_else(|| {
            panic!(
                "No underlying asset address found for the given liquidation pair: {}",
                liquidation_pair,
            )
        })
}

/// Get the asset decimals for a given asset on a specific network
pub fn get_asset_decimals(chain_id: U64, asset: Address) -> u8 {
    *ASSET_DECIMALS
        .get(&chain_id)
        .unwrap_or_else(|| {
            panic!(
                "No asset decimals found for the given chain ID: {}",
                chain_id,
            )
        })
        .get(&asset)
        .unwrap_or_else(|| panic!("No decimals found for the given asset: {}", asset,))
}

/// Get the asset symbol for a given asset on a specific network
pub fn get_asset_symbol(chain_id: U64, asset: Address) -> &'static str {
    ASSET_SYMBOLS
        .get(&chain_id)
        .unwrap_or_else(|| {
            panic!(
                "No asset symbols found for the given chain ID: {}",
                chain_id,
            )
        })
        .get(&asset)
        .unwrap_or_else(|| panic!("No symbol found for the given asset: {}", asset,))
}
