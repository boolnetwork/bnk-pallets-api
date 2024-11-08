use crate::BoolSubClient;
use sp_core::H256 as Hash;

pub async fn evm_chain_id(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Option<u64>, subxt::Error> {
    let store = crate::bool::storage().evm_chain_id().chain_id();
    sub_client.query_storage(store, at_block).await
}
