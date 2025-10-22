#[subxt::subxt(runtime_metadata_path = "./metadata-new.scale")]
pub mod polkadot {}

use sp_keyring::AccountKeyring;
use subxt::{tx::PairSigner, OnlineClient, PolkadotConfig};
use subxt::utils::AccountId32;
use sp_core::Pair;
use std::env;

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        panic!("invalid params");
    }
    let threads: usize = args[1].parse().unwrap();
    let mut thread_transaction: usize = usize::MAX;
    if let Some(v) = args.get(2) {
        thread_transaction = v.parse().unwrap();
    }

    let mut send_accs: Vec<sp_core::sr25519::Pair> = vec![];
    let mut recv_accs: Vec<sp_core::sr25519::Pair> = vec![];
    for i in 0..threads {
        send_accs.push(AccountKeyring::numeric(i));
        recv_accs.push(AccountKeyring::numeric(i + 10000));
    }

    // Create a new API client, configured to talk to Polkadot nodes.
    let api = OnlineClient::<PolkadotConfig>::new().await.unwrap();
    let from = PairSigner::new(AccountKeyring::Alice.pair());
    let mut account_nonce = api.tx().account_nonce(&AccountId32::from(AccountKeyring::Alice.pair().public())).await.unwrap();
    let mut progresses = vec![];
    for acc in &[send_accs.clone(), recv_accs.clone()].concat() {
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
        let _hash = progress.wait_for_in_block().await.unwrap().extrinsic_hash();
        // println!("Alice transfer to {dest:?} hash {hash:?}");
    }

    let mut tasks: Vec<tokio::task::JoinHandle<u32>> = vec![];
    let start = std::time::Instant::now();
    for i in 0..threads {
        let from = send_accs[i].clone();
        let dest = recv_accs[i].clone();
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
            // Build a balance transfer extrinsic.
            let dest = AccountId32::from(dest.public());
            let balance_transfer_tx = polkadot::tx().balances().transfer(dest.into(), 10_000);
            let mut account_nonce = 0u32;
            let mut tx_counter = 0usize;
            loop {
                // Submit the balance transfer extrinsic from Alice, and wait for it to be successful
                // and in a finalized block. We get back the extrinsic events if all is well.
                api
                    .tx()
                    .create_signed_with_nonce(&balance_transfer_tx, &PairSigner::new(from.clone()), account_nonce, Default::default())
                    .unwrap();
                    // .submit()
                    // .await
                    // .unwrap();
                // .submit_and_watch()
                // .await
                // .unwrap()
                // .wait_for_in_block()
                // .await
                // .unwrap();
                tx_counter += 1;
                account_nonce += 1;
                if tx_counter >= thread_transaction {
                    break account_nonce;
                }
            }
        });
        tasks.push(task);
    }
    let mut total = 0u128;
    for task in tasks {
        total += task.await.unwrap() as u128;
    }
    let time = start.elapsed().as_millis();
    println!("average: {} tps", total * 1000 / time);
}
