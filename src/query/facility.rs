use sp_core::H256 as Hash;
use crate::BoolSubClient;

pub async fn hash_to_version(
    sub_client: &BoolSubClient,
    version: u16,
    at_block: Option<Hash>,
) -> Option<Vec<u8>> {
    let store = crate::bool::storage().facility().hash_to_version(&version);
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query hash info for version: {}", version);
            }
            res
        },
        Err(e) => {
            log::error!(target: "pallets_api", "query hash failed: version: {} for {:?}", version, e);
            return None;
        }
    }
}

pub async fn version_list(sub_client: &BoolSubClient, at_block: Option<Hash>) -> Option<Vec<u16>> {
    let store = crate::bool::storage().facility().version_list();
    match sub_client.query_storage_or_default(store, at_block).await {
        Ok(res) => Some(res),
        Err(e) => {
            log::error!(target: "pallets_api", "query version_list failed for {:?}", e);
            return None;
        }
    }
}
