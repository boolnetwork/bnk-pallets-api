use sp_core::H256 as Hash;
use crate::BoolSubClient;

pub async fn block_hash(
    sub_client: &BoolSubClient,
    height: u32,
    at_block: Option<Hash>,
) -> Result<Option<Hash>, subxt::Error> {
    let storage_query = crate::bool::storage().system().block_hash(height);
    sub_client.query_storage(storage_query, at_block).await
}
