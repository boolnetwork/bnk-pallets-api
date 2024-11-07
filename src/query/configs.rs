use crate::BoolSubClient;
use sp_core::H256 as Hash;

pub async fn round_msg_wait(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Option<u64>, subxt::Error> {
    let store = crate::bool::storage().configs().round_msg_wait();
    sub_client.query_storage(store, at_block).await
}

pub async fn monitor_delay_tolerance(
    sub_client: &BoolSubClient,
    chain_id: u32,
    at_block: Option<Hash>,
) -> Result<u64, subxt::Error> {
    let store = crate::bool::storage().configs().monitor_delay_tolerance(chain_id);
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn monitor_delay_tolerance_iter(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Vec<(u32, u64)>, subxt::Error> {
    let store = crate::bool::storage().configs().monitor_delay_tolerance_root();
    sub_client
        .query_storage_value_iter(store, 300, at_block)
        .await
        .map(|res| {
            res.into_iter()
                .map(|(key, v)| {
                    let mut cid_bytes = [0u8; 4];
                    cid_bytes.copy_from_slice(&key.0[48..]);
                    (u32::from_le_bytes(cid_bytes), v)
                })
                .collect()
        })
}

pub async fn device_url_map(
    sub_client: &BoolSubClient,
    id: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<Vec<u8>>, subxt::Error> {
    let storage_query = crate::bool::storage().configs().device_url_map(id);
    sub_client.query_storage(storage_query, at_block).await
}
