use sp_core::H256 as Hash;
use crate::bool::runtime_types::pallet_channel::types::{HandleConnection, TxSource, CmtType, TaprootType, XudtStatus};
use crate::{BoolSubClient, handle_custom_error};

pub async fn create_channel(
    client: &BoolSubClient,
    info: Vec<u8>,
    connections: Vec<HandleConnection>,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().create_channel(info, connections);
    client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
}

pub async fn bind_committees(
    client: &BoolSubClient,
    channel_id: u32,
    connections: Vec<HandleConnection>,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().bind_committees(channel_id, connections);

    client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
}

pub async fn submit_transaction(
    client: &BoolSubClient,
    channel_id: u32,
    cid: u32,
    msg: Vec<u8>,
    source: TxSource,
    need_watch_res: bool,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().import_new_tx(channel_id, cid, msg, source);
    if need_watch_res {
        client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
    } else {
        client.submit_extrinsic_with_signer_without_watch(call, nonce).await.map_err(|e| e.to_string())
    }
}

pub async fn import_new_src_hash(
    client: &BoolSubClient,
    cid: u32,
    hash: Vec<u8>,
    src_chain_id: u32,
    uid: Vec<u8>,
    need_watch_res: bool,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().import_new_source_hash(cid, hash, src_chain_id, uid);
    if need_watch_res {
        client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
    } else {
        client.submit_extrinsic_with_signer_without_watch(call, nonce).await.map_err(|e| e.to_string())
    }
}

pub async fn report_result(
    client: &BoolSubClient,
    pk: Vec<u8>,
    sig: Vec<u8>,
    cid: u32,
    fork_id: u8,
    hash: Hash,
    signature: Vec<u8>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().submit_tx_sign_result(pk, sig, cid, fork_id, hash, signature);
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}

pub async fn report_result_call_bytes(
    client: &BoolSubClient,
    pk: Vec<u8>,
    sig: Vec<u8>,
    cid: u32,
    fork_id: u8,
    hash: Hash,
    signature: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let call = crate::bool::tx().channel().submit_tx_sign_result(pk, sig, cid, fork_id, hash, signature);
    client.unsigned_tx_encode_to_bytes(call).await.map_err(handle_custom_error)
}

pub async fn request_sign(
    client: &BoolSubClient,
    cid: u32,
    hash: Hash,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().request_sign(cid, hash);
    client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
}

pub async fn sync_status(
    client: &BoolSubClient,
    cid: u32,
    hash: Vec<u8>,
    watch_res: bool,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().sync_status(cid, hash);
    if watch_res {
        client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
    } else {
        client.submit_extrinsic_with_signer_without_watch(call, nonce).await.map_err(|e| e.to_string())
    }
}

pub async fn clear_target_package(
    client: &BoolSubClient,
    cid: u32,
    package_key: Vec<u8>,
    watch_res: bool,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().clear_target_package(cid, package_key);
    if watch_res {
        client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
    } else {
        client.submit_extrinsic_with_signer_without_watch(call, nonce).await.map_err(|e| e.to_string())
    }
}

pub async fn create_channel_with_taproot(
    client: &BoolSubClient,
    info: Vec<u8>,
    connections: Vec<(u32, u32, Vec<u8>, CmtType)>,
    taproot_types: Vec<(u32, TaprootType)>,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().create_channel_with_taproot(info, connections, taproot_types);
    client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
}

pub async fn request_to_sign_refresh(
    client: &BoolSubClient,
    cid: u32,
    inscription_tx: Vec<u8>,
    inscription_pos: u8,
    msg: Vec<u8>,
    watch_res: bool,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().request_to_sign_refresh(cid, inscription_tx, inscription_pos, msg);
    if watch_res {
        client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
    } else {
        client.submit_extrinsic_with_signer_without_watch(call, nonce).await.map_err(|e| e.to_string())
    }
}

pub async fn submit_refresh_result(
    client: &BoolSubClient,
    cid: u32,
    inscription_tx: Vec<u8>,
    inscription_pos: u8,
    sender_pk: Vec<u8>,
    sender_sig: Vec<u8>,
    cmt_sig: Vec<u8>,
    fork_id: u8,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().submit_refresh_result(
        cid,
        inscription_tx,
        inscription_pos,
        sender_pk,
        sender_sig,
        cmt_sig,
        fork_id,
    );
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}

pub async fn submit_refresh_result_call_bytes(
    client: &BoolSubClient,
    cid: u32,
    inscription_tx: Vec<u8>,
    inscription_pos: u8,
    sender_pk: Vec<u8>,
    sender_sig: Vec<u8>,
    cmt_sig: Vec<u8>,
    fork_id: u8,
) -> Result<Vec<u8>, String> {
    let call = crate::bool::tx().channel().submit_refresh_result(
        cid,
        inscription_tx,
        inscription_pos,
        sender_pk,
        sender_sig,
        cmt_sig,
        fork_id,
    );
    client.unsigned_tx_encode_to_bytes(call).await.map_err(handle_custom_error)
}

pub async fn sign_issue_xudt(
    client: &BoolSubClient,
    cid: u32,
    args_of_token: Vec<u8>,
    msg: Vec<u8>,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().sign_issue_xudt(
        cid,
        args_of_token,
        msg,
    );
    client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
}

pub async fn submit_issue_xudt_sign_result(
    client: &BoolSubClient,
    cid: u32,
    args_of_token: Vec<u8>,
    pk: Vec<u8>,
    sig: Vec<u8>,
    fork_id: u8,
    signature: Vec<u8>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().submit_issue_xudt_sign_result(
        cid,
        args_of_token,
        pk,
        sig,
        fork_id,
        signature,
    );
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}

pub async fn submit_issue_xudt_sign_result_call_bytes(
    client: &BoolSubClient,
    cid: u32,
    args_of_token: Vec<u8>,
    pk: Vec<u8>,
    sig: Vec<u8>,
    fork_id: u8,
    signature: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let call = crate::bool::tx().channel().submit_issue_xudt_sign_result(
        cid,
        args_of_token,
        pk,
        sig,
        fork_id,
        signature,
    );
    client.unsigned_tx_encode_to_bytes(call).await.map_err(handle_custom_error)
}

pub async fn sync_issue_xudt_result(
    client: &BoolSubClient,
    cid: u32,
    args_of_token: Vec<u8>,
    status: XudtStatus,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().sync_issue_xudt_result(
        cid,
        args_of_token,
        status,
    );
    client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
}

pub async fn update_src_hash_seq(
    client: &BoolSubClient,
    cid: u32, // dst_cid
    src_chain: u32,
    src_hash: Vec<u8>,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().update_src_hash_seq(
        cid,
        src_chain,
        src_hash,
    );
    client.submit_extrinsic_with_signer_without_watch(call, nonce).await.map_err(|e| e.to_string())
}

pub async fn submit_uid_sign_result(
    client: &BoolSubClient,
    cid: u32,
    uid: Vec<u8>,
    pk: Vec<u8>,
    sig: Vec<u8>,
    fork_id: u8,
    signature: Vec<u8>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().submit_uid_sign_result(
        cid,
        uid,
        pk,
        sig,
        fork_id,
        signature,
    );
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}

pub async fn submit_uid_sign_result_call_bytes(
    client: &BoolSubClient,
    cid: u32,
    uid: Vec<u8>,
    pk: Vec<u8>,
    sig: Vec<u8>,
    fork_id: u8,
    signature: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let call = crate::bool::tx().channel().submit_uid_sign_result(
        cid,
        uid,
        pk,
        sig,
        fork_id,
        signature,
    );
    client.unsigned_tx_encode_to_bytes(call).await.map_err(handle_custom_error)
}

pub async fn request_to_sign_forced_withdrawal(
    client: &BoolSubClient,
    tx_nonce: u128,
    msg: Vec<u8>,
    watch_res: bool,
    nonce: Option<u32>,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().sign_forced_withdrawal(tx_nonce, msg);
    if watch_res {
        client.submit_extrinsic_with_signer_and_watch(call, nonce).await.map_err(|e| e.to_string())
    } else {
        client.submit_extrinsic_with_signer_without_watch(call, nonce).await.map_err(|e| e.to_string())
    }
}
pub async fn finish_forced_withdrawal_result(
    client: &BoolSubClient,
    cid: u32,
    tx_nonce: u128,
    sender_pk: Vec<u8>,
    sender_sig: Vec<u8>,
    cmt_sig: Vec<u8>,
    fork_id: u8,
) -> Result<Hash, String> {
    let call = crate::bool::tx().channel().finish_forced_withdrawal(
        cid,
        tx_nonce,
        sender_pk,
        sender_sig,
        cmt_sig,
        fork_id,
    );
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}

pub async fn finish_forced_withdrawal_result_call_bytes(
    client: &BoolSubClient,
    cid: u32,
    tx_nonce: u128,
    sender_pk: Vec<u8>,
    sender_sig: Vec<u8>,
    cmt_sig: Vec<u8>,
    fork_id: u8,
) -> Result<Vec<u8>, String> {
    let call = crate::bool::tx().channel().finish_forced_withdrawal(
        cid,
        tx_nonce,
        sender_pk,
        sender_sig,
        cmt_sig,
        fork_id,
    );
    client.unsigned_tx_encode_to_bytes(call).await.map_err(handle_custom_error)
}
