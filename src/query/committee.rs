use crate::bool::runtime_types::fp_account::AccountId20;
use crate::bool::runtime_types::pallet_committee::types::{Committee, GlobalConfig};
use crate::BoolSubClient;
use sp_core::H256 as Hash;

pub async fn global_epoch(sub_client: &BoolSubClient, at_block: Option<Hash>) -> Result<u64, subxt::Error> {
    let store = crate::bool::storage().committee().global_epoch();
    sub_client.query_storage_or_default(store, at_block).await
}

pub async fn epoch_config(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<GlobalConfig<u32>, subxt::Error> {
    let store = crate::bool::storage().committee().epoch_config();
    sub_client.query_storage_or_default(store, at_block).await
}

pub async fn next_epoch_config(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Option<GlobalConfig<u32>>, subxt::Error> {
    let store = crate::bool::storage().committee().next_epoch_config();
    sub_client.query_storage(store, at_block).await
}

pub async fn committees(
    sub_client: &BoolSubClient,
    cid: u32,
    at_block: Option<Hash>,
) -> Result<Option<Committee<AccountId20, u32>>, subxt::Error> {
    let store = crate::bool::storage().committee().committees(cid);
    sub_client.query_storage(store, at_block).await
}

pub async fn committees_iter(
    sub_client: &BoolSubClient,
    page_size: u32,
    at_block: Option<Hash>,
) -> Result<Vec<Committee<AccountId20, u32>>, subxt::Error> {
    let store = crate::bool::storage().committee().committees_root();
    sub_client
        .query_storage_value_iter(store, page_size, at_block)
        .await
        .map(|res| {
            res.into_iter()
                .map(|v| v.1)
                .collect()
        })
}

pub async fn snapshot(sub_client: &BoolSubClient, at_block: Option<Hash>) -> Result<Vec<Vec<u8>>, subxt::Error> {
    let store = crate::bool::storage().committee().snapshot();
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn candidate_pool(sub_client: &BoolSubClient, at_block: Option<Hash>) -> Result<Vec<Vec<u8>>, subxt::Error> {
    let store = crate::bool::storage().committee().candidate_pool();
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn committee_members(
    sub_client: &BoolSubClient,
    cid: u32,
    epoch: u32,
    fork_id: u8,
    at_block: Option<Hash>,
) -> Result<Option<Vec<Vec<u8>>>, subxt::Error> {
    let store = crate::bool::storage().committee().committee_members(cid, (epoch, fork_id));
    sub_client.query_storage(store, at_block).await
}

pub async fn member_links(
    sub_client: &BoolSubClient,
    member: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<u32, subxt::Error> {
    let store = crate::bool::storage().committee().member_links(member);
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn member_links_iter(
    sub_client: &BoolSubClient,
    page_size: u32,
    at_block: Option<Hash>,
) -> Result<Vec<(Vec<u8>, u32)>, subxt::Error> {
    let store = crate::bool::storage().committee().member_links_root();
    sub_client
        .query_storage_value_iter(store, page_size, at_block)
        .await
        .map(|res| {
            res.into_iter()
                .map(|(k, v)| (k.0[49..].to_vec(), v))
                .collect()
        })
}

pub async fn candidate_links(
    sub_client: &BoolSubClient,
    cid: u32,
    fork: u8,
    at_block: Option<Hash>,
) -> Result<Vec<u16>, subxt::Error> {
    let store = crate::bool::storage().committee().candidate_links(cid, fork);
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn committee_randomness(
    sub_client: &BoolSubClient,
    cid: u32,
    at_block: Option<Hash>,
) -> Result<Option<u64>, subxt::Error> {
    let store = crate::bool::storage().committee().c_randomness(cid);
    sub_client.query_storage(store, at_block).await
}

pub async fn unpaid_sign_fee(
    sub_client: &BoolSubClient,
    pk: Vec<u8>,
    epoch: u32,
    at_block: Option<Hash>,
) -> Result<Option<u128>, subxt::Error> {
    let store = crate::bool::storage().committee().unpaid_sign_fee(pk, epoch);
    sub_client.query_storage(store, at_block).await
}

pub async fn identity_rewards(sub_client: &BoolSubClient, ident: Vec<u8>, at_block: Option<Hash>) -> Result<u128, subxt::Error> {
    let store = crate::bool::storage().committee().identity_rewards(ident);
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn exposed_identity(sub_client: &BoolSubClient, ident: Vec<u8>, at_block: Option<Hash>) -> Result<Vec<u8>, subxt::Error> {
    let store = crate::bool::storage().committee().exposed_identity(ident);
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn rewards_for_fork(
    sub_client: &BoolSubClient,
    cid: u32,
    epoch: u32,
    fork_id: u8,
    at_block: Option<Hash>,
) -> Result<Option<(u128, Vec<Vec<u8>>)>, subxt::Error> {
    let store = crate::bool::storage()
        .committee()
        .rewards_for_fork(cid, epoch, fork_id);
    sub_client.query_storage(store, at_block).await
}
