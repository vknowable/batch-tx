use std::str::FromStr;
use namada_sdk::{
    address::Address, args::{InputAmount, TxBuilder, TxTransparentTransferData}, io::NullIo, key::common::SecretKey, masp::fs::FsShieldedUtils, rpc, signing::default_sign, tx::ProcessTxResponse, wallet::fs::FsWalletUtils, Namada, NamadaImpl
};
use namada_token::Transfer;
use utils::{GAS_LIMIT, KEYPAIRS, NATIVE_TOKEN};
use crate::utils::{build_ctx, load_keys, load_transfer_targets};
use serde::Deserialize;
use tendermint_rpc::HttpClient;
use namada_sdk::borsh::BorshDeserialize;

pub mod utils;

#[derive(Debug, Deserialize, Clone)]
pub struct TransferTarget {
    address: &'static str,
    amount: u64,
}

async fn build_transfer_data(
    sdk: &NamadaImpl<HttpClient, FsWalletUtils, FsShieldedUtils, NullIo>,
    source: &str,
    target: &str,
    token: &Address,
    raw_amount: u64,
) -> TxTransparentTransferData {
    let source = sdk
        .wallet()
        .await
        .find_address(source)
        .unwrap()
        .into_owned();
    let target = Address::from_str(target).unwrap();
    let amount = InputAmount::from_str(raw_amount.to_string().as_str()).unwrap();

    TxTransparentTransferData {
        source,
        target,
        token: token.clone(),
        amount,
    }
}

#[tokio::main]
async fn main() {
    let sdk = build_ctx().await;

    // Wallet things
    load_keys(&sdk, &KEYPAIRS).await;

    let transfer_targets = load_transfer_targets();

    let native_token = Address::from_str(NATIVE_TOKEN).unwrap();
    let pubkey_0 = sdk.wallet().await.find_public_key("key-0").unwrap();
    let pubkey_1 = sdk.wallet().await.find_public_key("key-1").unwrap();

    let token = native_token;

    let mut data = Vec::new();

    // Define the transfers in the batch
    data.push(build_transfer_data(&sdk, "key-0", transfer_targets[0].address, &token, 1).await);
    data.push(build_transfer_data(&sdk, "key-0", transfer_targets[1].address, &token, 2).await);
    data.push(build_transfer_data(&sdk, "key-1", transfer_targets[2].address, &token, 2).await);
    println!("{:#?}", data);

    // Build the tx
    let mut transfer_tx_builder = sdk
        .new_transparent_transfer(data.clone())
        // .dry_run(true) // Uncomment for dry-run mode
        // .gas_limit(gas_limit.into()) // Uncomment if you want to set a different gas limit
        .signing_keys(vec![pubkey_0.clone(), pubkey_1.clone()]);
        
    let (mut transfer_tx, signing_data) = transfer_tx_builder
        .build(&sdk)
        .await
        .expect("unable to build transfer");

    // Deserialize and print the actual tx transfer data, after going through the tx builder, (for comparison)
    let presigned_tx = transfer_tx.clone();
    let data_bytes = presigned_tx.sections[1].data().unwrap().data;
    let data_deserialized = Transfer::try_from_slice(&data_bytes).unwrap();
    println!("{:#?}", data_deserialized);

    // Sign the tx
    sdk.sign(
        &mut transfer_tx,
        &transfer_tx_builder.tx,
        signing_data,
        default_sign,
        (),
    )
    .await
    .expect("unable to sign transparent-transfer tx");

    // Submit the tx
    match sdk.submit(transfer_tx.clone(), &transfer_tx_builder.tx).await {
        Ok(res) => {
            println!("Tx result: {:?}", res);
            if let ProcessTxResponse::Applied(applied) = res {
                // Print the tx hash
                println!("Tx hash: {}", applied.hash.to_string());
            }
        }
        Err(e) => println!("\n\nTx error: {:?}\n\n", e),
    }
    
    // Print some results out
    // for target in transfer_targets {
    //     let target = Address::from_str(&target.address).unwrap();
    //     let balance = rpc::get_token_balance(&sdk.client, &token, &target, None)
    //         .await
    //         .unwrap();
    //     println!("{}:  {}", &target, balance.to_string_native());
    // }

    // println!("{:#?}", transfer_tx);
    // println!("{:#?}", data);
}
