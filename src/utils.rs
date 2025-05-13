use namada_sdk::address::Address;
use namada_sdk::key::common::SecretKey;
use namada_sdk::{
    args::TxBuilder,
    chain::ChainId,
    io::NullIo,
    masp::{fs::FsShieldedUtils, ShieldedContext},
    wallet::fs::FsWalletUtils,
    Namada, NamadaImpl,
};
use serde::Deserialize;
use std::fmt::Debug;
use std::str::FromStr;
use namada_sdk::tendermint_rpc::{HttpClient, Url};
use std::fs::File;
use std::path::Path;

use crate::TransferTarget;

// Change these values depending on the chain you want to use
pub const CHAIN_ID: &str = "housefire-alpaca.cc0d3e0c033be";
pub const RPC_URL: &str = "https://rpc.housefire.tududes.com/";
pub const NATIVE_TOKEN: &str = "tnam1q9gr66cvu4hrzm0sd5kmlnjje82gs3xlfg3v6nu7";
pub const GAS_LIMIT: u64 = 500_000;

// CSV file paths
pub const KEYPAIRS_CSV: &str = "keypairs.csv";
pub const TRANSFER_TARGETS_CSV: &str = "transfer_targets.csv";

#[derive(Debug, Deserialize)]
pub struct AddressWithKey {
    pub address: String,
    pub secret_key: String,
}

// Load keypairs from CSV file
pub fn load_keypairs() -> Vec<AddressWithKey> {
    let file = File::open(KEYPAIRS_CSV).expect("Failed to open keypairs CSV file");
    let mut reader = csv::Reader::from_reader(file);
    
    reader.deserialize()
        .collect::<Result<Vec<AddressWithKey>, _>>()
        .expect("Failed to parse keypairs CSV")
}

// Load transfer targets from CSV file
pub fn load_transfer_targets() -> Vec<TransferTarget> {
    let file = File::open(TRANSFER_TARGETS_CSV).expect("Failed to open transfer targets CSV file");
    let mut reader = csv::Reader::from_reader(file);
    
    reader.deserialize()
        .collect::<Result<Vec<TransferTarget>, _>>()
        .expect("Failed to parse transfer targets CSV")
}

// Init the Namada chain context
pub async fn build_ctx() -> NamadaImpl<HttpClient, FsWalletUtils, FsShieldedUtils, NullIo>
{
    let rpc_url = RPC_URL;
    let chain_id = ChainId::from_str(CHAIN_ID).expect("Invalid chain ID");

    let url = Url::from_str(&rpc_url).expect("Invalid RPC address");
    let http_client = HttpClient::new(url).unwrap();

    // Init the wallet
    let wallet = FsWalletUtils::new("./sdk-wallet".into());

    // Init the shielded context
    let shielded_ctx = ShieldedContext::new(FsShieldedUtils::new("./masp".into()));

    let null_io = NullIo;

    // Init the Namada context
    NamadaImpl::new(http_client, wallet, shielded_ctx.into(), null_io)
        .await
        .expect("unable to initialize Namada context")
        .chain_id(chain_id)
}

// Load the keypairs into the wallet, must be done before signing the tx
pub async fn load_keys(
    sdk: &NamadaImpl<HttpClient, FsWalletUtils, FsShieldedUtils, NullIo>,
    keypairs: &[AddressWithKey],
) {
    for (idx, AddressWithKey { address, secret_key}) in keypairs.iter().enumerate() {
        println!("Loading keypair {} ({}) {}", idx, address, secret_key);
        let sk = SecretKey::from_str(secret_key).expect("Failed to parse secret key");
        let addr = Address::from_str(address).expect("Failed to parse address");

        sdk.wallet_mut()
            .await
            .insert_keypair(format!("key-{}", idx), false, sk, None, Some(addr), None)
            .expect("Failed to store keypair in wallet");
    }
}