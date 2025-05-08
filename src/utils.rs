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

use crate::TransferTarget;

// Change these values depending on the chain you want to use
pub const CHAIN_ID: &str = "campfire-square.ff09671d333707";
pub const RPC_URL: &str = "https://rpc.campfire.tududes.com/";
pub const NATIVE_TOKEN: &str = "tnam1qy440ynh9fwrx8aewjvvmu38zxqgukgc259fzp6h";
pub const GAS_LIMIT: u64 = 500_000;

// Our source/signing keypairs
pub static KEYPAIRS: [AddressWithKey; 3] = [
    AddressWithKey {
        address: "tnam1qphsmmw7g0vmcypczg7zrt6ngd9g049dyss6x6p2",
        secret_key: "009fada1a3e687e6b79ad17b6ad394ba27e5a227b12f6dedb1e2d4f8ff63d8e2b1",
    },
    AddressWithKey {
        address: "tnam1qzpmlg25zqh5mqjakktrnz2uyeaxrkhs5c839d40",
        secret_key: "001977660e163e253f9c2f62816d00dd332b8b8cec05755dcfff7247ebcee0416c",
    },
    AddressWithKey {
        address: "tnam1qrajwlyelry5ne9g4l7cjd9srmnxmr8x5ullvfyx",
        secret_key: "008b977a6acc41007bcd5758c049f1b766fc9da39f6fcbc1c2511b5331c478fada",
    },
];

// Our transfer targets (can be any valid addresses)
pub static TRANSFER_TARGETS: &[TransferTarget; 3] = &[
    TransferTarget {
        address: "tnam1qrtnphtj2yxdmafw24d9gqlkwaw70cpr8uv92ehr",
        amount: 0, // we don't use this field
    },
    TransferTarget {
        address: "tnam1qp3pljrm8mq6mz9jm6n802ceqtp9hal9nupxgtdw",
        amount: 0, // we don't use this field
    },
    TransferTarget {
        address: "tnam1qp8090ct234rjmpvwv8q8qpl5zdq929uqq9uew6e",
        amount: 0, // we don't use this field
    },
];

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

#[derive(Debug, Deserialize)]
pub struct AddressWithKey {
    address: &'static str,
    secret_key: &'static str,
}

// Load the keypairs into the wallet, must be done before signing the tx
pub async fn load_keys(
    sdk: &NamadaImpl<HttpClient, FsWalletUtils, FsShieldedUtils, NullIo>,
    keypairs: &[AddressWithKey],
) {
    for (idx, AddressWithKey { address, secret_key}) in keypairs.iter().enumerate() {
        // println!("{}: {}", address, secret_key);
        let sk = SecretKey::from_str(secret_key).expect("Failed to parse secret key");
        let addr = Address::from_str(address).expect("Failed to parse address");

        sdk.wallet_mut()
            .await
            .insert_keypair(format!("key-{}", idx), false, sk, None, Some(addr), None)
            .expect("Failed to store keypair in wallet");
    }
}

pub fn load_transfer_targets() -> Vec<TransferTarget> {
    TRANSFER_TARGETS.to_vec()
}