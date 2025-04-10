use sp_core::H256 as Hash;
use crate::{BoolSubClient, handle_custom_error};

pub async fn report_health(
    client: &BoolSubClient,
    ident: Vec<u8>,
    sig: Vec<u8>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().committee_health().report_health(ident, sig);
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}

pub async fn report_health_call_bytes(
    client: &BoolSubClient,
    ident: Vec<u8>,
    sig: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let call = crate::bool::tx().committee_health().report_health(ident, sig);
    client.unsigned_tx_encode_to_bytes(call).await.map_err(handle_custom_error)
}

pub async fn report_state_vote(
    client: &BoolSubClient,
    device_id: Vec<u8>,
    sig: Vec<u8>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().committee_health().report_state_vote(device_id, sig);
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}

pub async fn report_state_vote_call_bytes(
    client: &BoolSubClient,
    device_id: Vec<u8>,
    sig: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let call = crate::bool::tx().committee_health().report_state_vote(device_id, sig);
    client.unsigned_tx_encode_to_bytes(call).await.map_err(handle_custom_error)
}
