use soroban_sdk::{Env, panic_with_error, Symbol, Vec};

use crate::errors::ContractErrors;
use crate::storage::core::CoreData;

mod oracle {
    soroban_sdk::contractimport!(file = "oracle.wasm");
}

pub fn validate_asset(e: &Env, core_data: &CoreData, asset: &Symbol) {
    let oracle_client: oracle::Client = oracle::Client::new(&e, &core_data.oracle);
    let assets: Vec<oracle::Asset> = oracle_client.assets();
    let mut found: bool = false;
    for oracle_asset in assets.iter() {
        if oracle_asset == oracle::Asset::Other(asset.clone()) {
            found = true;
        }
    }

    if !found {
        panic_with_error!(&e, ContractErrors::InvalidAsset);
    }
}

pub fn get_latest_price(e: &Env, core_data: &CoreData, asset: &Symbol) -> oracle::PriceData {
    let oracle_client: oracle::Client = oracle::Client::new(&e, &core_data.oracle);
    let price_data: oracle::PriceData = oracle_client
        .lastprice(&oracle::Asset::Other(asset.clone()))
        .unwrap_or_else(|| {
            panic_with_error!(&e, &ContractErrors::AssetPriceNotFound);
        });

    price_data
}
