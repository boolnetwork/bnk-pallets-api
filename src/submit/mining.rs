use anyhow::Result;
use sp_core::H256 as Hash;
use subxt::ext::subxt_core::utils::AccountId20;
use crate::{BoolSubClient, handle_custom_error};
use crate::bool::runtime_types::pallet_mining::types::{OnChainPayload, MonitorType};

pub async fn im_online(client: &BoolSubClient, payload: OnChainPayload) -> Result<Hash, String> {
    let call = crate::bool::tx().mining().im_online(payload);
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}

pub async fn report_standby(
    client: &BoolSubClient,
    id: Vec<u8>,
    version: u16,
    enclave_hash: Vec<u8>,
    signature: Vec<u8>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().mining().report_standby(id, version, enclave_hash, signature);
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}

pub async fn register_device_with_ident(
    client: &BoolSubClient,
    owner: AccountId20,
    report: Vec<u8>,
    version: u16,
    identity: Vec<u8>,
    monitor_type: MonitorType,
    signature: Vec<u8>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().mining().register_device_with_ident(
        owner,
        report,
        version,
        identity,
        monitor_type,
        signature
    );
    let tx_process = client
        .submit_extrinsic_without_signer_and_watch(call)
        .await
        .map_err(|e| e.to_string())?;
    match tx_process.wait_for_finalized().await {
        Ok(tx) => Ok(tx.wait_for_success().await.map_err(|e| e.to_string())?.extrinsic_hash()),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn update_votes(
    client: &BoolSubClient,
    changed_votes: Vec<(Vec<u8>, u128)>,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().mining().update_votes(
        changed_votes,
    );
    client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
}

pub async fn join_service(
    client: &BoolSubClient,
    id: Vec<u8>,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().mining().join_service(id);
    client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
}

pub async fn exit_service(
    client: &BoolSubClient,
    id: Vec<u8>,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().mining().exit_service(id);
    client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
}
