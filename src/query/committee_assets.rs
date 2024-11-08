use crate::BoolSubClient;
use sp_core::H256 as Hash;

pub async fn all_concerned_brc20(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Option<Vec<Vec<u8>>>, subxt::Error> {
    let store = crate::bool::storage().committee_assets().all_concerned_brc20();
    sub_client.query_storage(store, at_block).await
}

pub async fn brc20_decimals(
    sub_client: &BoolSubClient,
    tick: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<u8>, subxt::Error> {
    let store = crate::bool::storage().committee_assets().brc20_decimals(tick);
    sub_client.query_storage(store, at_block).await
}

pub async fn committee_assets_consensus(
    sub_client: &BoolSubClient,
    cid: u32,
    at_block: Option<Hash>,
) -> Result<Option<(Vec<u16>, u64, Vec<u8>)>, subxt::Error> {
    let store = crate::bool::storage()
        .committee_assets()
        .committee_assets_consensus(cid);
    sub_client.query_storage(store, at_block).await
}
