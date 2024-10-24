use crate::BoolSubClient;
use sp_core::H256 as Hash;

pub async fn evm_chain_id(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Option<u64> {
    let store = crate::bool::storage().evm_chain_id().chain_id();
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query evm chain id return None");
            }
            res
        },
        Err(e) => {
            log::error!(target: "pallets_api", "query evm chain id failed for {:?}", e);
            return None;
        }
    }
}
