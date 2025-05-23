use crate::utils::{build_ctx, load_keypairs, load_keys, load_transfer_targets};
use namada_sdk::borsh::BorshDeserialize;
use namada_sdk::{
    Namada, NamadaImpl,
    address::Address,
    args::{InputAmount, TxBuilder, TxTransparentTransferData},
    io::NullIo,
    masp::fs::FsShieldedUtils,
    rpc,
    signing::default_sign,
    tx::ProcessTxResponse,
    wallet::fs::FsWalletUtils,
};
use namada_token::Transfer;
use serde::Deserialize;
use std::str::FromStr;
use tendermint_rpc::HttpClient;
use utils::{GAS_LIMIT, NATIVE_TOKEN};

pub mod utils;

#[derive(Debug, Deserialize, Clone)]
pub struct TransferTarget {
    pub address: String,
    pub amount: u64,
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
    let keypairs = load_keypairs();
    load_keys(&sdk, &keypairs).await;

    let transfer_targets = load_transfer_targets();

    let native_token = Address::from_str(NATIVE_TOKEN).unwrap();
    let pubkey_0 = sdk.wallet().await.find_public_key("key-0").unwrap();

    let token = native_token;

    // Get all source public keys for signing
    let mut signing_keys = Vec::new();
    for idx in 0..keypairs.len() {
        let key_name = format!("key-{}", idx);
        let pubkey = sdk.wallet().await.find_public_key(&key_name).unwrap();
        signing_keys.push(pubkey);
    }

    // Check if each source's public key has been revealed on chain
    println!("\nChecking if source public keys are revealed on chain:");
    let mut unrevealed_addresses = Vec::new();
    for (idx, keypair) in keypairs.iter().enumerate() {
        let address = Address::from_str(&keypair.address).unwrap();
        match rpc::is_public_key_revealed(&sdk.client, &address).await {
            Ok(is_revealed) => {
                println!(
                    "Source {} ({}): {}",
                    idx,
                    keypair.address,
                    if is_revealed {
                        "REVEALED"
                    } else {
                        "NOT REVEALED"
                    }
                );
                if !is_revealed {
                    unrevealed_addresses.push((idx, address));
                }
            }
            Err(e) => {
                println!(
                    "Source {} ({}): Error checking reveal status - {}",
                    idx, keypair.address, e
                );
            }
        }
    }
    println!(); // Add a blank line for readability

    // Handle unrevealed public keys one by one
    if !unrevealed_addresses.is_empty() {
        println!(
            "Found {} addresses with unrevealed public keys:",
            unrevealed_addresses.len()
        );
        for (idx, addr) in &unrevealed_addresses {
            println!("\nRevealing source {} ({}):", idx, addr);
            let key_name = format!("key-{}", idx);
            let key_to_reveal = sdk.wallet().await.find_public_key(&key_name).unwrap();
            // Build the reveal pk transaction
            let reveal_tx_builder = sdk
                .new_reveal_pk(key_to_reveal)
                .signing_keys(vec![pubkey_0.clone()]);
            let (mut reveal_tx, signing_data) = reveal_tx_builder
                .build(&sdk)
                .await
                .expect("unable to build reveal pk tx");

            // Sign the transaction
            sdk.sign(
                &mut reveal_tx,
                &reveal_tx_builder.tx,
                signing_data,
                default_sign,
                (),
            )
            .await
            .expect("unable to sign reveal pk tx");

            // Submit the signed tx to the ledger for execution
            // Assumes account already has funds to cover gas costs
            match sdk.submit(reveal_tx.clone(), &reveal_tx_builder.tx).await {
                Ok(res) => println!("Tx result: {:?}", res),
                Err(e) => println!("Tx error: {:?}", e),
            }
        }
        println!("\nAll reveal transactions processed. Proceeding with transfers...");
    }

    let mut data = Vec::new();

    // Create transfers by pairing each source key with its corresponding target
    for (idx, target) in transfer_targets.iter().enumerate() {
        let source_key = format!("key-{}", idx % keypairs.iter().count());
        data.push(
            build_transfer_data(&sdk, &source_key, &target.address, &token, target.amount).await,
        );
    }
    println!("{:#?}", data);

    // Build the tx
    let mut transfer_tx_builder = sdk
        .new_transparent_transfer(data.clone())
        // .dry_run(true) // Uncomment for dry-run mode
        // .gas_limit(GAS_LIMIT.into()) // Uncomment if you want to set a different gas limit
        .signing_keys(signing_keys);

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
    match sdk
        .submit(transfer_tx.clone(), &transfer_tx_builder.tx)
        .await
    {
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
