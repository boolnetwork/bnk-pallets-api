use sp_core::H256 as Hash;
use crate::bool::runtime_types::pallet_channel::types::{
    BtcCmtType, BtcTxTunnel, Channel, CommitteeFeeConfig, ForcedWithdrawalRecord, RefreshRecord, SourceTXInfo, TaprootPair, TxMessage, UidRecord, XudtInfo, XudtIssueRecord
};
use crate::bool::runtime_types::fp_account::AccountId20;
use crate::BoolSubClient;

pub async fn tx_messages(
    sub_client: &BoolSubClient,
    cid: u32,
    hash: Hash,
    at_block: Option<Hash>,
) -> Result<Option<TxMessage<u32>>, subxt::Error> {
    let store = crate::bool::storage().channel().tx_messages(cid, hash);
    sub_client.query_storage(store, at_block).await
}

pub async fn channel_info(
    sub_client: &BoolSubClient,
    channel_id: u32,
    at_block: Option<Hash>,
) -> Result<Option<Channel<AccountId20>>, subxt::Error> {
    let store = crate::bool::storage().channel().channel_info(channel_id);
    sub_client.query_storage(store, at_block).await
}

pub async fn hashes_for_cid(
    sub_client: &BoolSubClient,
    cid: u32,
    at_block: Option<Hash>,
) -> Result<Option<(Vec<SourceTXInfo>, BtcTxTunnel)>, subxt::Error> {
    let store = crate::bool::storage().channel().hashes_for_cid(cid);
    sub_client.query_storage(store, at_block).await
}

pub async fn source_tx_package(
    sub_client: &BoolSubClient,
    cid: u32,
    package_key: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<Vec<SourceTXInfo>>, subxt::Error> {
    let store = crate::bool::storage().channel().source_tx_package(cid, package_key.clone());
    sub_client.query_storage(store, at_block).await
}

pub async fn source_hash_to_package_key
(
    sub_client: &BoolSubClient,
    chain_id: u32,
    src_hash: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<Vec<u8>>, subxt::Error> {
    let store = crate::bool::storage().channel().source_hash_to_package_key(chain_id, src_hash.clone());
    sub_client.query_storage(store, at_block).await
}

pub async fn btc_committee_type(
    sub_client: &BoolSubClient,
    cid: u32,
    at_block: Option<Hash>,
) -> Result<Option<BtcCmtType>, subxt::Error> {
    let store = crate::bool::storage().channel().btc_committee_type(cid);
    sub_client.query_storage(store, at_block).await
}

pub async fn btc_committee_type_iter(
    sub_client: &BoolSubClient,
    page_size: u32,
    at_block: Option<Hash>,
) -> Result<Vec<BtcCmtType>, subxt::Error> {
    let store = crate::bool::storage().channel().btc_committee_type_root();
    sub_client
        .query_storage_value_iter(store, page_size, at_block)
        .await
        .map(|res| {
            res.into_iter()
                .map(|v| v.1)
                .collect()
        })
}

pub async fn escape_taproot(sub_client: &BoolSubClient, cid: u32, at_block: Option<Hash>) -> Result<Option<TaprootPair>, subxt::Error> {
    let store = crate::bool::storage().channel().escape_taproots(cid);
    sub_client.query_storage(store, at_block).await
}

pub async fn escape_taproot_iter(
    sub_client: &BoolSubClient,
    page_size: u32,
    at_block: Option<Hash>,
) -> Result<Vec<TaprootPair>, subxt::Error> {
    let store = crate::bool::storage().channel().escape_taproots_root();
    sub_client
        .query_storage_value_iter(store, page_size, at_block)
        .await
        .map(|res| {
            res.into_iter()
                .map(|v| v.1)
                .collect()
        })
}

pub async fn refresh_record(
    sub_client: &BoolSubClient,
    inscription_hash: Vec<u8>,
    inscription_pos: u8,
    at_block: Option<Hash>,
) -> Result<Option<RefreshRecord>, subxt::Error> {
    let store = crate::bool::storage().channel().refresh_data(inscription_hash.clone(), inscription_pos);
    sub_client.query_storage(store, at_block).await
}

pub async fn committee_xudt_list(
    sub_client: &BoolSubClient,
    cid: u32,
    at_block: Option<Hash>,
) -> Result<Option<Vec<XudtInfo>>, subxt::Error> {
    let store = crate::bool::storage().channel().committee_xudt_list(cid);
    sub_client.query_storage(store, at_block).await
}

pub async fn committee_xudt_record(
    sub_client: &BoolSubClient,
    cid: u32,
    args_of_token: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<XudtIssueRecord>, subxt::Error> {
    let store = crate::bool::storage().channel().committee_xudt_record(cid, args_of_token.clone());
    sub_client.query_storage(store, at_block).await
}

pub async fn uid_consensus_record(
    sub_client: &BoolSubClient,
    cid: u32,
    uid: Vec<u8>,
    at_block: Option<Hash>,
) -> Result<Option<UidRecord<u32>>, subxt::Error> {
    let store = crate::bool::storage().channel().uid_consensus_record(cid, uid.clone());
    sub_client.query_storage(store, at_block).await
}

pub async fn committee_fee_data(
    sub_client: &BoolSubClient,
    cid: u32,
    at_block: Option<Hash>,
) -> Result<Option<CommitteeFeeConfig>, subxt::Error> {
    let store = crate::bool::storage().channel().committee_fee_data(cid);
    sub_client.query_storage(store, at_block).await 
}

pub async fn committee_fee_data_iter(
    sub_client: &BoolSubClient,
    page_size: u32,
    at_block: Option<Hash>,
) -> Result<Vec<(u32, CommitteeFeeConfig)>, subxt::Error> {
    let store = crate::bool::storage().channel().committee_fee_data_root();
    sub_client
        .query_storage_value_iter(store, page_size, at_block)
        .await
        .map(|res| {
            res
                .into_iter()
                .map(|(key, value)| {
                    let mut cid_bytes = [0u8; 4];
                    cid_bytes.copy_from_slice(&key.0[48..]);
                    (u32::from_le_bytes(cid_bytes), value)
                })
                .collect()
        })
}

pub async fn channel_mapping_tick_iter(
    sub_client: &BoolSubClient,
    page_size: u32,
    at_block: Option<Hash>,
) -> Result<Vec<(u32, Vec<(Vec<u8>, Vec<u8>)>)>, subxt::Error> {
    let store = crate::bool::storage().channel().channel_mapping_tick_root();
    sub_client
        .query_storage_value_iter(store, page_size, at_block)
        .await
        .map(|res| {
            res
                .into_iter()
                .map(|(key, value)| {
                    let mut cid_bytes = [0u8; 4];
                    cid_bytes.copy_from_slice(&key.0[48..]);
                    (u32::from_le_bytes(cid_bytes), value)
                })
                .collect()
        })
}

pub async fn channel_mapping_tick(
    sub_client: &BoolSubClient,
    channel_id: u32,
    at_block: Option<Hash>,
) -> Result<Option<Vec<(Vec<u8>, Vec<u8>)>>, subxt::Error> {
    let store = crate::bool::storage().channel().channel_mapping_tick(channel_id);
    sub_client.query_storage(store, at_block).await
}

pub async fn forced_withdrawal_record(
    sub_client: &BoolSubClient,
    nonce_key: u128,
    at_block: Option<Hash>,
) -> Option<ForcedWithdrawalRecord> {
    let store = crate::bool::storage().channel().forced_withdrawal_data(nonce_key);
    match sub_client.query_storage(store, at_block).await {
        Ok(res) => {
            if res.is_none() {
                log::warn!(target: "pallets_api", "query none forced withdrawal data for nonce key: {:?}", nonce_key);
            }
            res
        },
        Err(e) => {
            log::error!(target: "pallets_api", "query forced withdrawal data failed for: {:?}", e);
            return None;
        }
    }
}