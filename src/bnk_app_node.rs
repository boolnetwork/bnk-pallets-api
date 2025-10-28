#[subxt::subxt(
    runtime_metadata_path = "./metadata-bnk-app-node.scale",
    derive_for_all_types = "Eq, PartialEq, Clone, Debug"
)]
pub mod polkadot {}

use sp_core::{Pair, H160, U256, H256 as Hash};
use std::env;
use bnk_node_primitives::AccountId20;
use codec::{Compact, Encode};
use subxt::rpc::types;
use subxt::{rpc_params, Config};
use bnk_pallets_api::{BoolConfig, BoolSubClient};
use bnk_pallets_api::bool::runtime_types::ethereum::transaction::{TransactionV2 as BTransaction, TransactionAction as BTransactionAction};
use polkadot::runtime_types::ethereum::transaction::{TransactionV2 as Transaction, EIP1559Transaction, TransactionAction};
use subxt::tx::{BoolSigner, SecretKey};
use subxt::OnlineClient;
use std::str::FromStr;

pub async fn transact(
    client: &BoolSubClient,
    transaction: Transaction,
    source: H160,
) -> Result<Hash, String> {
    let call = polkadot::tx().ethereum().transact(transaction, source);
    client.submit_extrinsic_without_signer(call).await.map_err(bnk_pallets_api::handle_custom_error)
}

pub async fn transacts(
    client: &BoolSubClient,
    transactions_and_sources: Vec<(H160, Transaction)>,
) -> Result<Vec<Hash>, String> {
    let calls = transactions_and_sources.into_iter().map(|(source, transaction)| polkadot::tx().ethereum().transact(transaction, source)).collect();
    client.submit_extrinsics_without_signer(calls).await.map_err(bnk_pallets_api::handle_custom_error)
}

#[derive(Copy, Clone, Debug)]
pub enum TransferType {
    Native,
    Erc20(H160),
}

pub const TRANSFER_SELECTOR: [u8; 4] = [169, 5, 156, 187];

pub async fn transfer_by_evm(
    sub_client: &BoolSubClient,
    chain_id: u64,
    dst: &[u8],
    amount: u128,
    transfer_type: TransferType,
    nonce: u32,
) -> Result<polkadot::runtime_types::ethereum::transaction::TransactionV2, String> {
    use precompile_utils::solidity::codec::Writer as EvmDataWriter;

    let (input, call_to, value) = match transfer_type {
        TransferType::Native => (vec![], H160::from_slice(&dst), U256::from(amount)),
        TransferType::Erc20(token) => {
            let input = EvmDataWriter::new_with_selector(u32::from_be_bytes(TRANSFER_SELECTOR))
                .write(precompile_utils::prelude::Address::from(H160::from_slice(&dst)))
                .write(amount)
                .build();
            (input, token, U256::from(0u128))
        }
    };
    let tx = ethereum::EIP1559Transaction {
        chain_id,
        nonce: sp_core::U256::from(nonce),
        max_priority_fee_per_gas: sp_core::U256::from(1500000000u128),
        max_fee_per_gas: sp_core::U256::from(4500000000u128),
        gas_limit: sp_core::U256::from(500000u128),
        action: ethereum::TransactionAction::Call(call_to),
        value,
        input,
        access_list: Default::default(),
        odd_y_parity: false,
        r: Default::default(),
        s: Default::default()
    };
    let tx = sub_client.build_eip1559_tx_to_v2(tx.clone())?;
    Ok(
        match tx {
            BTransaction::EIP1559(tx) => {
                Transaction::EIP1559(EIP1559Transaction {
                    chain_id: tx.chain_id,
                    nonce: polkadot::runtime_types::primitive_types::U256(
                        tx.nonce.0,
                    ),
                    max_priority_fee_per_gas: polkadot::runtime_types::primitive_types::U256(
                        tx.max_priority_fee_per_gas.0
                    ),
                    max_fee_per_gas: polkadot::runtime_types::primitive_types::U256(
                        tx.max_fee_per_gas.0
                    ),
                    gas_limit: polkadot::runtime_types::primitive_types::U256(
                        tx.gas_limit.0
                    ),
                    action: match tx.action {
                        BTransactionAction::Call(addr) => TransactionAction::Call(addr),
                        _ => return Err(format!("Invalid evm tx action: {:?}", tx.action))
                    },
                    value: polkadot::runtime_types::primitive_types::U256(
                        tx.value.0
                    ),
                    input: tx.input,
                    access_list: vec![],
                    odd_y_parity: tx.odd_y_parity,
                    r: tx.r,
                    s: tx.s,
                })
            }
            _ => return Err("Invalid evm tx".to_string()),
        }
    )
}

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        panic!("invalid params");
    }
    match args[1].as_str() {
        "storage_prefix" => storage_prefix(args),
        "transfer" => transfer(args).await,
        "ethereum_transfer" => ethereum_transfer(args).await,
        "tree_speed" => test_tree_speed(args),
        _ => panic!("invalid mission"),
    }
}



fn storage_prefix(args: Vec<String>) {
    if args.len() < 4 {
        panic!("invalid params");
    }
    let pallet_hash = sp_io::hashing::twox_128(args[2].as_bytes());
    let storage_hash = sp_io::hashing::twox_128(args[3].as_bytes());

    let mut final_key = [0u8; 32];
    final_key[..16].copy_from_slice(&pallet_hash);
    final_key[16..].copy_from_slice(&storage_hash);
    println!("{:?}", hex::encode(final_key));
    println!("{final_key:?}");
}

async fn ethereum_transfer(args: Vec<String>) {
    if args.len() < 4 {
        panic!("invalid params");
    }
    let mut url = "ws://127.0.0.1:9944".to_string();
    let mut i = 2usize;
    if args[i].starts_with("ws://") || args[i].starts_with("wss://") {
        url = args[i].parse().unwrap();
        i += 1;
    }
    let threads: usize = args[i].parse().unwrap();
    i += 1;
    let thread_transaction: usize = args[i].parse().unwrap();
    i += 1;
    let mut submit_batch_size: usize = 1;
    if args.len() >= i + 1 {
        submit_batch_size = args[i].parse().unwrap();
        i += 1;
        submit_batch_size = submit_batch_size.max(1);
    }
    let mut loop_times = 1u8;
    if args.len() >= i + 1 {
        loop_times = args[i].parse().unwrap();
    }
    let transfer_type = TransferType::Native;
    // let transfer_type = match args[4].clone().as_str() {
    //     "native" => bnk_pallets_api::monitor_rpc::TransferType::Native,
    //     _ => {
    //         if args[2].len() != 20 {
    //             panic!("invalid transfer_type");
    //         }
    //         bnk_pallets_api::monitor_rpc::TransferType::Erc20(H160::from_slice(args[2].as_bytes()))
    //     }
    // };

    let mut send_accs: Vec<sp_core::ecdsa::Pair> = vec![];
    // let mut recv_accs: Vec<sp_core::ecdsa::Pair> = vec![];
    for i in 0..threads {
        let mut seed_alice = vec![10u8; 32];
        seed_alice[..8].copy_from_slice(&(i as u64).to_be_bytes());
        send_accs.push(sp_core::ecdsa::Pair::from_seed_slice(seed_alice.as_slice()).unwrap());
    }
    let chain_id = 483u64;
    let alice_sk = "5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133".to_string();
    let client = BoolSubClient::new_from_ecdsa_sk(url.clone(), Some(alice_sk), None, None).await.unwrap();
    let signer = client.signer.as_ref().unwrap();
    let account_id = signer.account_id();
    let mut account_nonce = client.client.read().await.tx().account_nonce(&account_id).await.unwrap();
    for acc in &send_accs.clone() {
        let dest: AccountId20 = AccountId20::from(acc.public());
        let balance_transfer_tx = transfer_by_evm(
            &client,
            chain_id,
            dest.0.as_slice(),
            20_000_000_000_000_000_000,
            transfer_type,
            account_nonce,
        )
            .await
            .unwrap();
        account_nonce += 1;
        let _hash = transact(&client, balance_transfer_tx, H160::from(account_id.0)).await.unwrap();
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    for i in 1..=loop_times {
        println!("start round{i}");
        let mut tasks: Vec<tokio::task::JoinHandle<usize>> = vec![];
        let total_start = std::sync::Arc::new(tokio::sync::RwLock::new(None));
        let prepare_count = std::sync::Arc::new(tokio::sync::RwLock::new(0usize));
        for i in 0..threads {
            let total_start_i = total_start.clone();
            let prepare_count_i = prepare_count.clone();
            let from = send_accs[i].clone();
            let url = url.clone();
            let task = tokio::spawn(async move {
                let sk = hex::encode(from.seed().as_slice());
                let client = BoolSubClient::new_from_ecdsa_sk(url, Some(sk.clone()), None, None).await.unwrap();
                let storage_query = polkadot::storage().system().account(&polkadot::runtime_types::fp_account::AccountId20(AccountId20::from(from.public()).0));
                let result = client
                    .client
                    .read()
                    .await
                    .storage()
                    .at_latest()
                    .await
                    .unwrap()
                    .fetch(&storage_query)
                    .await
                    .unwrap();
                println!("Alice{i} sk {sk} adress {} has free balance: {}", AccountId20::from(from.public()), result.unwrap().data.free);

                let signer = client.signer.as_ref().unwrap();
                let account_id = signer.account_id();
                let mut account_nonce = client.client.read().await.tx().account_nonce(&account_id).await.unwrap();
                // Build a balance transfer extrinsic.
                let mut transactions = vec![];
                let prepare_start = std::time::Instant::now();
                println!("Alice{i} start prepare transactions");
                for t in 0..thread_transaction {
                    let mut seed_bob = vec![11u8; 32];
                    seed_bob[..8].copy_from_slice(&(i as u64 * 100000 + 10000 + t as u64).to_be_bytes());
                    let dest = sp_core::ecdsa::Pair::from_seed_slice(seed_bob.as_slice()).unwrap();
                    let dest = AccountId20::from(dest.public());
                    transactions.push((
                        t,
                        // Submit the balance transfer extrinsic from Alice, and wait for it to be successful
                        // and in a finalized block. We get back the extrinsic events if all is well.
                        transfer_by_evm(
                            &client,
                            chain_id,
                            dest.0.as_slice(),
                            10_000,
                            transfer_type,
                            account_nonce,
                        )
                            .await
                            .unwrap(),
                        account_nonce,
                    ));
                    account_nonce += 1;
                }
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
                let mut txs = 0;
                for (chunk_i, chunk) in transactions.chunks(submit_batch_size).enumerate() {
                    if submit_batch_size == 1 {
                        for (tx_i, transaction, nonce) in chunk.to_vec() {
                            let tx_number = tx_i + 1;
                            if let Err(e) = transact(&client, transaction, H160::from(account_id.0)).await {
                                println!("alice{i} nonce: {nonce} Error: {e:?}");
                                break;
                            }
                            txs += 1;
                            if txs % 100 == 0 {
                                println!("alice{i} tx: {tx_number}, tps: {}", (tx_number) as u128 * 1000 / start.elapsed().as_millis());
                            }
                        }
                    } else {
                        let mut tx_number = 0;
                        let mut nonce = 0;
                        let calls: Vec<_> = chunk
                            .to_vec()
                            .into_iter()
                            .map(|(tx_i, transaction, n)| {
                                tx_number = tx_i + 1;
                                nonce = n;
                                (H160::from(account_id.0), transaction)
                            })
                            .collect();
                        if let Err(e) = transacts(&client, calls).await {
                            println!("alice{i} nonce: {nonce} chunk: {chunk_i} Error: {e:?}");
                            break;
                        }
                        txs += chunk.len();
                        let time = start.elapsed().as_micros();
                        println!("alice{i} chunk: {chunk_i} tx: {tx_number} time: {time} micros, tps: {}", (tx_number) as u128 * 1000_000 / time);
                    }
                }
                let time = start.elapsed().as_micros();
                println!("alice{i} total tx: {} time: {time} micros, tps: {}", txs, txs as u128 * 1000_1000 / time);
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

async fn transfer(args: Vec<String>) {
    use rand::{thread_rng, Rng};


    if args.len() < 4 {
        panic!("invalid params");
    }

    let mut url = "ws://127.0.0.1:9944".to_string();

    let mut rng = thread_rng();
    if args[2].starts_with("ws://") || args[2].starts_with("wss://") {
        url = args[2].parse().unwrap();
    }
    let threads: usize = args[3].parse().unwrap();
    let thread_transaction: usize = args[4].parse().unwrap();
    let mut submit_batch_size: usize = 1;
    if args.len() >= 5 {
        submit_batch_size = args[5].parse().unwrap();
        submit_batch_size = submit_batch_size.max(1);
    }
    let mut loop_times = 1u8;
    if args.len() >= 6 {
        loop_times = args[6].parse().unwrap();
    }

    let mut final_dest = Vec::new();
    for i in 0..threads {
        let mut thread_dest = Vec::new();
        for j in 0..thread_transaction {
            let dest: BoolSigner<BoolConfig> = BoolSigner::new(SecretKey::random(&mut rng));
            thread_dest.push(dest.clone());
        }
        final_dest.push(thread_dest);
    }

    let mut send_accs: Vec<BoolSigner<BoolConfig>> = vec![];
    for i in 0..threads {
        // let sk = [i as u8; 32];
        let signer = BoolSigner::new(SecretKey::random(&mut rng));
        send_accs.push(signer);
        // send_accs.push(AccountKeyring::numeric(i));
    }
    // Create a new API client, configured to talk to Polkadot nodes.

    let from = {
        let sk = hex::decode("5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133").unwrap();
        let signer = BoolSigner::new(SecretKey::parse_slice(&sk).unwrap());
        signer
    };
    let client = BoolSubClient::new_from_signer(&url, Some(from.clone()), None, None).await.unwrap();

    let mut account_nonce = client.client.read().await.tx().account_nonce(&AccountId20::from_str("0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac").unwrap()).await.unwrap();
    let mut progresses = vec![];
    for acc in &send_accs.clone() {
        let dest = polkadot::runtime_types::fp_account::AccountId20 { 0: acc.account_id().0 };

        let balance_transfer_tx = polkadot::tx().balances().transfer_allow_death(dest.clone().into(), 10_000_000_000_000);

        let progress = client.client.read().await.tx().create_signed_with_nonce(&balance_transfer_tx, &from, account_nonce, Default::default())
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
            let threads_dest = final_dest[i].clone();
            let url = url.clone();
            let task = tokio::spawn(async move {
                let from_acc = polkadot::runtime_types::fp_account::AccountId20 { 0: from.account_id().0 };
                let client = BoolSubClient::new_from_signer(&url, Some(from.clone()), None, None).await.unwrap();
                // let api = OnlineClient::<BoolConfig>::from_url(&url).await.unwrap();
                let storage_query = polkadot::storage().system().account(from_acc);
                let result = client
                    .client
                    .read()
                    .await
                    .storage()
                    .at_latest()
                    .await
                    .unwrap()
                    .fetch(&storage_query)
                    .await
                    .unwrap();

                println!("Alice{i} adress {} has free balance: {}", from.account_id(), result.unwrap().data.free);
                let mut account_nonce = client
                    .client
                    .read()
                    .await
                    .tx()
                    .account_nonce(&from.account_id())
                    .await
                    .unwrap();
                let prepare_start = std::time::Instant::now();
                let mut transactions = Vec::new();
                for t in 0..thread_transaction {
                    // Submit the balance transfer extrinsic from Alice, and wait for it to be successful
                    // and in a finalized block. We get back the extrinsic events if all is well.

                    let dest = threads_dest[t].clone();
                    // let dest: BoolSigner<BoolConfig> = BoolSigner::new(SecretKey::random(&mut rng));
                    let dest = polkadot::runtime_types::fp_account::AccountId20 { 0: dest.account_id().0 };

                    // let dest = AccountId32::from(AccountKeyring::numeric(i * 10000  + 100000 + t).public());
                    let balance_transfer_tx = polkadot::tx().balances().transfer_allow_death(dest.into(), 10_000);
                    let tx = client
                        .client
                        .read()
                        .await
                        .tx()
                        .create_signed_with_nonce(&balance_transfer_tx, &from.clone(), account_nonce, Default::default())
                        .unwrap();
                    account_nonce += 1;

                    transactions.push((t, tx, account_nonce - 1));
                }
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
                        if let Err(e) = client
                            .client
                            .read()
                            .await.rpc().request::<Vec<<BoolConfig as Config>::Hash>>("author_submitExtrinsics", params).await {
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

fn test_tree_speed(args: Vec<String>) {
    let mut key_length_pairs = vec![];
    if args.len() < 6 {
        panic!("invalid args length < 5 (from, to, step, repeat)");
    }
    let from: usize = args[2].parse().unwrap();
    let to: usize = args[3].parse().unwrap();
    let step: usize = args[4].parse().unwrap();
    let repeat: usize = args[5].parse().unwrap();
    let mut i = 0;
    loop {
        let pairs = from + i * step;
        if pairs > to {
            break;
        }
        key_length_pairs.push(pairs);
        i += 1;
    }
    let mut write_times = vec![];
    let mut read_times = vec![];

    for pairs in key_length_pairs.clone() {
        let keys = (0..pairs).map(|i| {
            let mut seed_bob = vec![11u8; 32];
            seed_bob[..8].copy_from_slice(&(i as u64 * 100000 + 10000).to_be_bytes());
            let dest = sp_core::ecdsa::Pair::from_seed_slice(seed_bob.as_slice()).unwrap();
            let dest = AccountId20::from(dest.public()).0.to_vec();
            let prefix = vec![1u8; 32];
            [prefix, dest.clone()].concat()
        }).collect::<Vec<_>>();

        let mut btree_insert_time = 0u128;
        let mut btree_read_time = 0u128;
        let mut hashmap_insert_time = 0u128;
        let mut hashmap_read_time = 0u128;
        for _ in 0..repeat {
            let mut btree = std::collections::BTreeMap::new();
            for k in keys.clone() {
                btree.insert(k.clone(), k);
            }
            let mut hashmap = std::collections::HashMap::with_capacity(pairs);
            for k in keys.clone() {
                hashmap.insert(k.clone(), k);
            }
            for i in 0..pairs {
                let k = keys[i].clone();

                let start = std::time::Instant::now();
                btree.insert(k.clone(), k.clone());
                btree_insert_time += start.elapsed().as_nanos();
                let start = std::time::Instant::now();
                let _ = btree.get(&k.clone());
                btree_read_time += start.elapsed().as_nanos();
                btree.remove(&k);

                let start = std::time::Instant::now();
                hashmap.insert(k.clone(), k.clone());
                hashmap_insert_time += start.elapsed().as_nanos();
                let start = std::time::Instant::now();
                let _ = hashmap.get(&k.clone());
                hashmap_read_time += start.elapsed().as_nanos();
                hashmap.remove(&k);
            }
        }
        btree_insert_time /= repeat as u128;
        hashmap_insert_time /= repeat as u128;
        btree_read_time /= repeat as u128;
        hashmap_read_time /= repeat as u128;
        write_times.push(((btree_insert_time, btree_insert_time / pairs as u128), (hashmap_insert_time, hashmap_insert_time / pairs as u128)));
        read_times.push(((btree_read_time, btree_read_time / pairs as u128), (hashmap_read_time, hashmap_read_time / pairs as u128)));
    }
    println!("Write Times:");
    for (i, pairs) in key_length_pairs.iter().enumerate() {
        let write_time = &write_times[i];
        println!("{pairs:5}: btree: {:7}({:3}), hashmap: {:7}({:3})", write_time.0.0, write_time.0.1, write_time.1.0, write_time.1.1);
    }
    println!("Read Times:");
    for (i, pairs) in key_length_pairs.iter().enumerate() {
        let read_time = &read_times[i];
        println!("{pairs:5}: btree: {:7}({:3}), hashmap: {:7}({:3})", read_time.0.0, read_time.0.1, read_time.1.0, read_time.1.1);
    }
}
