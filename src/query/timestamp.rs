use sp_core::H256 as Hash;
use crate::BoolSubClient;

pub async fn now(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Option<u64>, subxt::Error> {
    let storage_query = crate::bool::storage().timestamp().now();
    sub_client.query_storage(storage_query, at_block).await
}
