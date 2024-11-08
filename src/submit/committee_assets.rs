use sp_core::H256 as Hash;
use crate::{BoolSubClient, handle_custom_error};

pub async fn update_assets(
    client: &BoolSubClient,
    cid: u32,
    block_number: u32,
    btc_asset: u128,
    brc20_assets: Vec<(Vec<u8>, u128)>,
    sender_pk: Vec<u8>,
    sender_sig: Vec<u8>,
    cmt_sig: Vec<u8>,
    fork_id: u8,
) -> Result<Hash, String> {
    let call = crate::bool::tx().committee_assets().update_assets(cid, block_number, btc_asset, brc20_assets, sender_pk, sender_sig, cmt_sig, fork_id);
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}
