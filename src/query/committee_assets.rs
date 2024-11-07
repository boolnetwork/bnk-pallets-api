use crate::BoolSubClient;
use sp_core::H256 as Hash;

pub async fn all_concerned_brc20(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Option<Vec<Vec<u8>>> {
    let store = crate::bool::storage().committee_assets().all_concerned_brc20();
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
    let store = crate::bool::storage().committee_assets().brc20_decimals(tick);
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
        .committee_assets()
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
