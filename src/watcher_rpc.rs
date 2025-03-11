#![allow(clippy::type_complexity)]

//! relay service for key server.
use codec::Encode;
use crate::bool::runtime_types::{
    pallet_facility::pallet::DIdentity,
    pallet_mining::types::{OnChainPayload, Purpose, MonitorType},
    ethereum::transaction::{EIP1559Transaction, TransactionV2 as Transaction, TransactionAction},
};
use crate::query::mining::{working_devices, challenges};
use crate::query::ethereum::evm_chain_id;
use crate::submit::mining::{im_online, register_device_with_ident};
use crate::submit::ethereum::{transact_unsigned, transact_unsigned_call_bytes};
use crate::BoolSubClient;
use crate::no_prefix;
use sp_core::{H160, H256};
use precompile_utils::solidity::codec::Writer as EvmDataWriter;
use precompile_utils::prelude::UnboundedBytes;

/// keccak_256("submitTxSignResult(bytes[],bytes[],uint256,uint256,bytes32,bytes[])".as_bytes())[..4]
pub const REPORT_RESULT_SELECTOR: [u8; 4] = [118, 72, 134, 178];
/// keccak_256("importNewTx(uint256,uint256,bytes[],uint256,bytes[],bytes[],bytes[],uint256)".as_bytes())[..4]
pub const SUBMIT_TRANSACTION_SELECTOR: [u8; 4] = [58, 164, 61, 2];
/// keccak_256("joinOrExitServiceUnsigned(bytes[],uint256,bytes[],bytes[])".as_bytes())[..4]
pub const JOIN_OR_EXIT_SERVICE_UNSIGNED_SELECTOR: [u8; 4] = [99, 254, 70, 76];

pub async fn call_register_v2(
    sub_client: &BoolSubClient,
    config_owner: &str,
    did: (u16, Vec<u8>),
    report: Vec<u8>,
    identity: Vec<u8>,
    monitor_type: MonitorType,
    signature: Vec<u8>,
) -> Result<String, String> {
    let (version, _pk) = did;
    let owner = hex::decode(no_prefix(config_owner)).map_err(|e| e.to_string())?;
    let mut owner_bytes = [0u8; 20];
    owner_bytes.copy_from_slice(&owner);
    match register_device_with_ident(
        sub_client,
        crate::bool::runtime_types::fp_account::AccountId20(owner_bytes),
        report,
        version,
        identity,
        monitor_type,
        signature,
    ).await {
        Ok(hash) => Ok("0x".to_string() + &hex::encode(hash.0)),
        Err(e) => Err(e),
    }
}

pub async fn call_heartbeat(
    sub_client: &BoolSubClient,
    did: (u16, Vec<u8>),
    signature: Vec<u8>,
    proof: Vec<u8>,
    session: u32,
    enclave: Vec<u8>,
) -> Result<String, String> {
    let did = DIdentity {
        version: did.0,
        pk: did.1,
    };
    let payload = OnChainPayload {
        did,
        proof,
        session,
        signature,
        enclave,
    };
    match im_online(sub_client, payload).await {
        Ok(hash) => Ok("0x".to_string() + &hex::encode(hash.0)),
        Err(e) => Err(e),
    }
}

pub async fn query_session_and_challenge(
    sub_client: &BoolSubClient,
    did: (u16, Vec<u8>),
) -> Result<Option<(u32, Vec<u8>)>, String> {
    let did = DIdentity {
        version: did.0,
        pk: did.1,
    };
    let (devices, session) = working_devices(sub_client, None, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("no working device".to_string())?;
    let res = if devices.contains(&(did, false)) {
        match challenges(sub_client, session, None).await.map_err(|e| e.to_string())? {
            Some(challenges) => Some((session, challenges.encode())),
            None => None,
        }
    } else {
        None
    };
    Ok(res)
}

pub async fn report_result_by_evm(
    sub_client: &BoolSubClient,
    pk: Vec<u8>,
    sig: Vec<u8>,
    cid: u32,
    fork_id: u8,
    hash: sp_core::H256,
    signature: Vec<u8>,
    call_bytes: bool,
) -> Result<Vec<u8>, String> {
    // build writer with 'reportResult' select
    let writer = EvmDataWriter::new_with_selector(u32::from_be_bytes(REPORT_RESULT_SELECTOR))
        .write(UnboundedBytes::from(pk))
        .write(UnboundedBytes::from(sig))
        .write(cid)
        .write(fork_id)
        .write(hash)
        .write(UnboundedBytes::from(signature));

    let input = writer.build();

    let chain_id = evm_chain_id(sub_client, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("get evm chain failed".to_string())?;
    let tx = ethereum::EIP1559TransactionMessage {
        chain_id,
        nonce: sp_core::U256::from(0u128),
        max_priority_fee_per_gas: sp_core::U256::from(1500000000u128),
        max_fee_per_gas: sp_core::U256::from(4500000000u128),
        gas_limit: sp_core::U256::from(50000000u128),
        action: ethereum::TransactionAction::Call(H160::from_low_u64_be(1104)),
        value: sp_core::U256::from(0u128),
        input,
        access_list: Default::default(),
    };
    let transaction = Transaction::EIP1559(EIP1559Transaction {
        chain_id,
        nonce: crate::bool::runtime_types::primitive_types::U256(
            tx.nonce.0,
        ),
        max_priority_fee_per_gas: crate::bool::runtime_types::primitive_types::U256(
            tx.max_priority_fee_per_gas.0
        ),
        max_fee_per_gas: crate::bool::runtime_types::primitive_types::U256(
            tx.max_fee_per_gas.0
        ),
        gas_limit: crate::bool::runtime_types::primitive_types::U256(
            tx.gas_limit.0
        ),
        // channel precompile contract address
        action: TransactionAction::Call(H160::from_low_u64_be(1104)),
        value: crate::bool::runtime_types::primitive_types::U256(
            tx.value.0
        ),
        input: tx.input,
        access_list: vec![],
        odd_y_parity: Default::default(),
        r: H256(
            Default::default()
        ),
        s: H256(
            Default::default()
        ),
    });

    if call_bytes {
        transact_unsigned_call_bytes(sub_client, transaction).await
    } else {
        transact_unsigned(sub_client, transaction)
            .await
            .map(|hash| hash.0.to_vec())
    }
}

pub async fn join_or_exit_service_unsigned_by_evm(
    sub_client: &BoolSubClient,
    id: Vec<u8>,
    msg: Vec<u8>,
    signature: Vec<u8>,
    purpose: Purpose,
) -> Result<String, String> {
    // build writer with select
    let writer = EvmDataWriter::new_with_selector(u32::from_be_bytes(JOIN_OR_EXIT_SERVICE_UNSIGNED_SELECTOR))
        .write(UnboundedBytes::from(id))
        .write(purpose as u8)
        .write(UnboundedBytes::from(msg))
        .write(UnboundedBytes::from(signature));

    let input = writer.build();

    let chain_id = evm_chain_id(sub_client, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("get evm chain failed".to_string())?;
    let tx = ethereum::EIP1559TransactionMessage {
        chain_id,
        nonce: sp_core::U256::from(0u128),
        max_priority_fee_per_gas: sp_core::U256::from(1500000000u128),
        max_fee_per_gas: sp_core::U256::from(4500000000u128),
        gas_limit: sp_core::U256::from(50000000u128),
        // mining precompile contract address
        action: ethereum::TransactionAction::Call(H160::from_low_u64_be(1101)),
        value: sp_core::U256::from(0u128),
        input,
        access_list: Default::default(),
    };
    let transaction = Transaction::EIP1559(EIP1559Transaction {
        chain_id,
        nonce: crate::bool::runtime_types::primitive_types::U256(
            tx.nonce.0,
        ),
        max_priority_fee_per_gas: crate::bool::runtime_types::primitive_types::U256(
            tx.max_priority_fee_per_gas.0
        ),
        max_fee_per_gas: crate::bool::runtime_types::primitive_types::U256(
            tx.max_fee_per_gas.0
        ),
        gas_limit: crate::bool::runtime_types::primitive_types::U256(
            tx.gas_limit.0
        ),
        action: TransactionAction::Call(H160::from_low_u64_be(1101)),
        value: crate::bool::runtime_types::primitive_types::U256(
            tx.value.0
        ),
        input: tx.input,
        access_list: vec![],
        odd_y_parity: Default::default(),
        r: H256(
            Default::default()
        ),
        s: H256(
            Default::default()
        ),
    });

    transact_unsigned(sub_client, transaction)
        .await
        .map(|hash| "0x".to_string() + &hex::encode(hash.0))
}

pub async fn query_current_block_number(sub_client: &BoolSubClient) -> Result<u32, String> {
    sub_client
        .client
        .read()
        .await
        .blocks()
        .at_latest()
        .await
        .map(|block| block.number())
        .map_err(|e| e.to_string())
}
