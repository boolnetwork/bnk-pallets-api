use crate::BoolSubClient;
use crate::bool::runtime_types::pallet_committee_health::pallet::{ConsensusStage, ConfirmData, DHCState};
use sp_core::H256 as Hash;

pub async fn identity_challenge(
    sub_client: &BoolSubClient,
    identity: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<(u32, Vec<u8>), subxt::Error> {
    let store = crate::bool::storage().committee_health().identity_challenge(identity);
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn court_members(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Option<Vec<Vec<u8>>>, subxt::Error> {
    let store = crate::bool::storage().committee_health().court_members();
    sub_client.query_storage(store, at_block).await
}

pub async fn consensus_state(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Option<DHCState>, subxt::Error> {
    let store = crate::bool::storage().committee_health().consensus_state();
    sub_client.query_storage(store, at_block).await
}

pub async fn state_votes(
    sub_client: &BoolSubClient,
    device_id: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Vec<u8>, subxt::Error> {
    let store = crate::bool::storage().committee_health().state_votes(device_id);
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn consensus_confirms(
    sub_client: &BoolSubClient,
    epoch: u64,
    stage: ConsensusStage,
    at_block: Option<Hash>,
) -> Result<Option<ConfirmData>, subxt::Error> {
    let store = crate::bool::storage().committee_health().consensus_confirms(epoch, stage);
    sub_client.query_storage(store, at_block).await
}

pub async fn submit_devices_whitelist(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Vec<Vec<u8>>, subxt::Error> {
    let store = crate::bool::storage().committee_health().submit_devices_whitelist();
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn submit_devices(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<Vec<Vec<u8>>, subxt::Error> {
    let store = crate::bool::storage().committee_health().submit_devices();
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}

pub async fn submit_devices_size(
    sub_client: &BoolSubClient,
    at_block: Option<Hash>,
) -> Result<u16, subxt::Error> {
    let store = crate::bool::storage().committee_health().submit_devices_size();
    sub_client.query_storage(store, at_block).await.map(|r| r.unwrap_or_default())
}
