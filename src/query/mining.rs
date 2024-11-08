use crate::bool::runtime_types::{
    primitive_types::U256,
    fp_account::AccountId20,
    pallet_facility::pallet::DIdentity,
    pallet_mining::types::{DeviceInfo, MonitorState},
};
use crate::BoolSubClient;
use sp_core::H256 as Hash;

pub async fn challenges(
    sub_client: &BoolSubClient,
    session: u32,
    at_block: Option<Hash>,
) -> Result<Option<U256>, subxt::Error> {
    let store = crate::bool::storage().mining().challenges(session);
    sub_client.query_storage(store, at_block).await
}

pub async fn working_devices(
    sub_client: &BoolSubClient,
    session: Option<u32>,
    at_block: Option<Hash>,
) -> Result<Option<(Vec<(DIdentity, bool)>, u32)>, subxt::Error> {
    let session = match session {
        Some(session) => session,
        None => {
            let client = sub_client.client.read().await.blocks();
            let current_block = match at_block {
                Some(hash) => {
                    client.at(hash).await
                },
                None => {
                    client.at_latest().await
                }
            };
            let current_number = current_block.map(|b| b.number())?;
            let constant_query = crate::bool::constants().mining().era_block_number();
            sub_client.query_constant(constant_query)
                .await
                .map(|era_block_number| current_number / era_block_number)?
        }
    };
    let store = crate::bool::storage().mining().working_devices(session);
    sub_client.query_storage(store, at_block)
        .await
        .map(|res| res.and_then(|data| Some((data, session))))
}

pub async fn device_info(
    sub_client: &BoolSubClient,
    id: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<DeviceInfo<AccountId20, u32, u128>>, subxt::Error> {
    let storage_query = crate::bool::storage().mining().devices(id.clone());
    sub_client.query_storage(storage_query, at_block).await
}

pub async fn device_info_iter(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Vec<DeviceInfo<AccountId20, u32, u128>>, subxt::Error> {
    let storage_query = crate::bool::storage().mining().devices_root();
    sub_client.query_storage_value_iter(storage_query, 300, at_block)
        .await
        .map(|res| {
            res.into_iter()
                .map(|v| v.1)
                .collect()
        })
}

pub async fn device_identity_map(
    sub_client: &BoolSubClient,
    id: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<Vec<u8>>, subxt::Error> {
    let storage_query = crate::bool::storage().mining().device_identity_map(id);
    sub_client.query_storage(storage_query, at_block).await
}

pub async fn device_monitor_state(
    sub_client: &BoolSubClient,
    id: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<MonitorState>, subxt::Error> {
    let storage_query = crate::bool::storage().mining().device_monitor_state(id);
    sub_client.query_storage(storage_query, at_block).await
}

pub async fn device_votes_for_current_epoch(
    sub_client: &BoolSubClient,
    id: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<Vec<(AccountId20, u128)>>, subxt::Error> {
    let storage_query = crate::bool::storage().mining().device_votes_for_current_epoch(id.clone());
    sub_client.query_storage(storage_query, at_block).await
}

pub async fn device_votes_for_next_epoch(
    sub_client: &BoolSubClient,
    id: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<Vec<(AccountId20, u128)>>, subxt::Error> {
    let storage_query = crate::bool::storage().mining().device_votes_for_next_epoch(id.clone());
    sub_client.query_storage(storage_query, at_block).await
}

pub async fn device_data(
    sub_client: &BoolSubClient,
    did: DIdentity,
    at_block: Option<Hash>,
) -> Result<Option<Vec<u8>>, subxt::Error> {
    let store = crate::bool::storage().mining().device_data(did.clone());
    sub_client.query_storage(store, at_block).await
}
