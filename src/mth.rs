#[subxt::subxt(runtime_metadata_path = "./metadata-new.scale")]
pub mod polkadot {}

use sp_keyring::AccountKeyring;
use subxt::{rpc_params, tx::PairSigner, Config, OnlineClient, PolkadotConfig};
use subxt::utils::AccountId32;
use sp_core::Pair;
use std::env;
use codec::{Compact, Encode};
use subxt::rpc::types;
use bnk_pallets_api::BoolConfig;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        panic!("invalid params");
    }
    match args[1].as_str() {
        "storage_prefix" => storage_prefix(args),
        "transfer" => transfer(args).await,
        _ => panic!("invalid mission"),
    }
}

fn storage_prefix(args: Vec<String>) {
    if args.len() < 4 {
        panic!("invalid params");
    }
    let storages = args[3..].to_vec();
    let pallet_hash = sp_io::hashing::twox_128(args[2].as_bytes());
    for storage in storages {
        let storage_hash = sp_io::hashing::twox_128(storage.as_bytes());
        let mut final_key = [0u8; 32];
        final_key[..16].copy_from_slice(&pallet_hash);
        final_key[16..].copy_from_slice(&storage_hash);
        println!("{} {storage}\n    {:?}\n    {final_key:?}", args[2], hex::encode(final_key));
    }
}

async fn transfer(args: Vec<String>) {
    if args.len() < 4 {
        panic!("invalid params");
    }
    let threads: usize = args[2].parse().unwrap();
    let thread_transaction: usize = args[3].parse().unwrap();
    let mut submit_batch_size: usize = 1;
    if args.len() >= 5 {
        submit_batch_size = args[4].parse().unwrap();
        submit_batch_size = submit_batch_size.max(1);
    }
    let mut loop_times = 1u8;
    if args.len() >= 6 {
        loop_times = args[5].parse().unwrap();
    }

    let mut send_accs: Vec<sp_core::sr25519::Pair> = vec![];
    for i in 0..threads {
        send_accs.push(AccountKeyring::numeric(i));
    }
    // Create a new API client, configured to talk to Polkadot nodes.
    let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();
    let from = PairSigner::new(AccountKeyring::Alice.pair());
    let mut account_nonce = api.tx().account_nonce(&AccountId32::from(AccountKeyring::Alice.pair().public())).await.unwrap();
    let mut progresses = vec![];
    for acc in &send_accs.clone() {
        let dest: AccountId32 = AccountId32::from(acc.public());
        let balance_transfer_tx = polkadot::tx().balances().transfer(dest.clone().into(), 10_000_000_000_000);
        let progress = api.tx().create_signed_with_nonce(&balance_transfer_tx, &from, account_nonce, Default::default())
            .unwrap()
            .submit_and_watch()
            .await
            .unwrap();
        account_nonce += 1;
        progresses.push(progress);
        if progresses.len() == 1000 {
            for progress in std::mem::take(&mut progresses) {
                let _hash = progress.wait_for_in_block().await.unwrap().extrinsic_hash();
                // println!("Alice transfer to {dest:?} hash {hash:?}");
            }
        }
    }
    for progress in progresses {
        let hash = progress.wait_for_in_block().await.unwrap().extrinsic_hash();
        println!("Alice transfer hash {hash:?}");
    }
    for i in 1..=loop_times {
        println!("start round{i}");
        let mut tasks: Vec<tokio::task::JoinHandle<usize>> = vec![];
        let total_start = std::sync::Arc::new(tokio::sync::RwLock::new(None));
        let prepare_count = std::sync::Arc::new(tokio::sync::RwLock::new(0usize));
        for i in 0..threads {
            let total_start_i = total_start.clone();
            let prepare_count_i = prepare_count.clone();
            let from = send_accs[i].clone();
            let task = tokio::spawn(async move {
                let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();
                let storage_query = polkadot::storage().system().account(&AccountId32::from(from.public()));
                let result = api
                    .storage()
                    .at_latest()
                    .await
                    .unwrap()
                    .fetch(&storage_query)
                    .await
                    .unwrap();

                println!("Alice{i} adress {} has free balance: {}", AccountId32::from(from.public()), result.unwrap().data.free);
                let mut account_nonce = api.tx().account_nonce(&AccountId32::from(from.public())).await.unwrap();
                let prepare_start = std::time::Instant::now();
                let transactions: Vec<_> = (0..thread_transaction).map(|t| {
                    // Submit the balance transfer extrinsic from Alice, and wait for it to be successful
                    // and in a finalized block. We get back the extrinsic events if all is well.
                    let dest = AccountId32::from(AccountKeyring::numeric(i * 10000  + 100000 + t).public());
                    let balance_transfer_tx = polkadot::tx().balances().transfer(dest.into(), 10_000);
                    let tx = api
                        .tx()
                        .create_signed_with_nonce(&balance_transfer_tx, &PairSigner::new(from.clone()), account_nonce, Default::default())
                        .unwrap();
                    account_nonce += 1;
                    (t, tx, account_nonce - 1)
                })
                    .collect();
                println!("Alice{i} prepare {thread_transaction} transactions in {} micros", prepare_start.elapsed().as_micros());
                *prepare_count_i.write().await += 1;
                loop {
                    if *prepare_count_i.read().await == threads {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_micros(10)).await;
                }
                let mut total_start_i = total_start_i.write().await;
                if total_start_i.is_none() {
                    *total_start_i = Some(std::time::Instant::now());
                }
                drop(total_start_i);
                let start = std::time::Instant::now();
                let mut txs = thread_transaction;
                if submit_batch_size == 1 {
                    for (tx_i, transactions, nonce) in transactions {
                        let tx_number = tx_i + 1;
                        if let Err(e) = transactions.submit().await {
                            txs = tx_number;
                            println!("alice{i} nonce: {nonce} Error: {e:?}");
                            break;
                        }
                        if tx_number % 100 == 0 {
                            println!("alice{i} tx: {tx_number}, tps: {}", (tx_number) as u128 * 1000 / start.elapsed().as_millis());
                        }
                    }
                } else {
                    for (chunk_i, chunk) in transactions.chunks(submit_batch_size).enumerate() {
                        let mut tx_number = 0;
                        let mut nonce = 0;
                        let calls: Vec<_> = chunk
                            .iter()
                            .map(|(tx_i, transaction, n)| {
                                tx_number = tx_i + 1;
                                nonce = *n;
                                transaction.encoded().to_vec()
                            })
                            .collect::<Vec<_>>()
                            .concat();
                        let mut encoded_txs = Vec::new();
                        Compact(chunk.len() as u32).encode_to(&mut encoded_txs);
                        encoded_txs.extend(calls);
                        let bytes: types::Bytes = encoded_txs.into();
                        let params = rpc_params![bytes];
                        if let Err(e) = api.rpc().request::<Vec<<BoolConfig as Config>::Hash>>("author_submitExtrinsics", params).await {
                            println!("alice{i} nonce: {nonce} chunk: {chunk_i} Error: {e:?}");
                            break;
                        } else {
                            txs += chunk.len();
                        }
                        let time = start.elapsed().as_micros();
                        println!("alice{i} chunk: {chunk_i} tx: {tx_number} time: {time} micros, tps: {}", (tx_number) as u128 * 1000_000 / time);
                    }
                }
                let time = start.elapsed().as_micros();
                println!("alice{i} tx: {} time: {time} micros, tps: {} ", i + 1, txs as u128 * 1000_000 / time);
                txs
            });
            tasks.push(task);
        }
        let mut total = 0;
        for task in tasks {
            total += task.await.unwrap();
        }
        let time = total_start.read().await.unwrap().elapsed().as_micros();
        println!("Total tx: {total} time: {time} micros, tps: {}", total as u128 * 1000_000 / time);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}
