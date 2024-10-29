use crate::bool::runtime_types::fp_account::AccountId20;
use crate::bool::runtime_types::pallet_committee::pallet::{Committee, GlobalConfig};
use crate::BoolSubClient;
use anyhow::anyhow;
use sp_core::H256 as Hash;

pub async fn global_epoch(sub_client: &BoolSubClient, at_block: Option<Hash>) -> anyhow::Result<u64> {
    let store = crate::bool::storage().committee().global_epoch();
    sub_client
        .query_storage_or_default(store, at_block)
        .await
        .map_err(|e| anyhow!("{e:?}"))
}

pub async fn epoch_config(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> anyhow::Result<GlobalConfig<u32>> {
    let store = crate::bool::storage().committee().epoch_config();
    sub_client
        .query_storage_or_default(store, at_block)
        .await
        .map_err(|e| anyhow!("{e:?}"))
}

pub async fn next_epoch_config(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> anyhow::Result<Option<GlobalConfig<u32>>> {
    let store = crate::bool::storage().committee().next_epoch_config();
    sub_client
        .query_storage(store, at_block)
        .await
        .map_err(|e| anyhow!("{e:?}"))
}

pub async fn pool_rate(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> anyhow::Result<u8> {
    let store = crate::bool::storage().committee().pool_rate();
    sub_client
        .query_storage_or_default(store, at_block)
        .await
        .map_err(|e| anyhow!("{e:?}"))
}

pub async fn committees(
    sub_client: &BoolSubClient,
    cid: u32,
    at_block: Option<Hash>,
) -> Option<Committee<AccountId20, u32>> {
    let store = crate::bool::storage().committee().committees(cid);
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query committee info for cid: {}", cid);
            }
            res
        }
        Err(e) => {
            log::error!(target: "pallets_api", "query committee failed: cid: {} for {:?}", cid, e);
            return None;
        }
    }
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
) -> Option<Vec<Vec<u8>>> {
    let store = crate::bool::storage()
        .committee()
        .committee_members(cid, (epoch, fork_id));
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query none members for cid: {}, epoch: {}, fork_id: {}", cid, epoch, fork_id);
            }
            res
        }
        Err(e) => {
            log::error!(target: "pallets_api", "query members failed:cid: {}, epoch: {}, fork_id: {}, for {:?}", cid, epoch, fork_id, e);
            return None;
        }
    }
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
) -> Option<u64> {
    let store = crate::bool::storage().committee().c_randomness(cid);
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query none randomness for cid: {}", cid);
            }
            res
        }
        Err(e) => {
            log::error!(target: "pallets_api", "query randomness failed: cid: {}, for {:?}", cid, e);
            return None;
        }
    }
}

pub async fn unpaid_sign_fee(
    sub_client: &BoolSubClient,
    pk: Vec<u8>,
    epoch: u32,
    at_block: Option<Hash>,
) -> Result<Option<u128>, String> {
    let store = crate::bool::storage()
        .committee()
        .unpaid_sign_fee(pk.clone(), epoch);
    let pk = "0x".to_string() + &hex::encode(&pk);
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => Ok(res),
        Err(e) => {
            log::error!(target: "pallets_api", "query unpaid_sign_fee failed for pk: {pk}, epoch: {epoch}, for: {e:?}");
            Err(format!(
                "query unpaid_sign_fee failed for pk: {pk}, epoch: {epoch}, for: {e:?}"
            ))
        }
    }
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
) -> Option<(u128, Vec<Vec<u8>>)> {
    let store = crate::bool::storage()
        .committee()
        .rewards_for_fork(cid, epoch, fork_id);
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query none rewards for cid: {}, epoch: {}, fork_id: {}", cid, epoch, fork_id);
            }
            res
        }
        Err(e) => {
            log::error!(target: "pallets_api", "query rewards failed for cid: {}, epoch: {}, fork_id: {}, for: {:?}", cid, epoch, fork_id, e);
            return None;
        }
    }
}

pub async fn all_concerned_brc20(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Option<Vec<Vec<u8>>> {
    let store = crate::bool::storage().committee().all_concerned_brc20();
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query none brc20 list");
            }
            res
        }
        Err(e) => {
            log::error!(target: "pallets_api", "query brc20 list failed for {:?}", e);
            return None;
        }
    }
}

pub async fn brc20_decimals(
    sub_client: &BoolSubClient,
    tick: Vec<u8>,
    at_block: Option<Hash>,
) -> Option<u8> {
    let store = crate::bool::storage().committee().brc20_decimals(tick);
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query none brc20 decimal");
            }
            res
        }
        Err(e) => {
            log::error!(target: "pallets_api", "query brc20 decimal failed for {:?}", e);
            return None;
        }
    }
}

pub async fn committee_assets_consensus(
    sub_client: &BoolSubClient,
    cid: u32,
    at_block: Option<Hash>,
) -> Option<(Vec<u16>, u64, Vec<u8>)> {
    let store = crate::bool::storage()
        .committee()
        .committee_assets_consensus(cid);
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query none committee asset consensus");
            }
            res
        }
        Err(e) => {
            log::error!(target: "pallets_api", "query committee asset consensus failed for {:?}", e);
            return None;
        }
    }
}
