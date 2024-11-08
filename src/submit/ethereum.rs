use sp_core::H256 as Hash;
use crate::{BoolSubClient, handle_custom_error};
use crate::bool::runtime_types::ethereum::transaction::TransactionV2 as Transaction;

pub async fn transact(
    client: &BoolSubClient,
    transaction: Transaction,
) -> Result<Hash, String> {
    let call = crate::bool::tx().ethereum().transact(transaction);
    client.submit_extrinsic_without_signer(call).await.map_err(|e| e.to_string())
}

pub async fn transact_unsigned(
    client: &BoolSubClient,
    transaction: Transaction,
) -> Result<Hash, String> {
    let call = crate::bool::tx().ethereum().transact_unsigned(transaction);
    client.submit_extrinsic_without_signer(call).await.map_err(handle_custom_error)
}
