use std::collections::HashMap;
use anyhow::Result;
use bnk_node_primitives::AccountId20;
use sp_core::H256 as Hash;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use subxt::config::{
    polkadot::PolkadotExtrinsicParams,
    substrate::{BlakeTwo256, SubstrateHeader},
};
use subxt::{
    OnlineClient, Config, tx::{BoolSigner, Payload as TxPayload, TxProgress, SecretKey, Signer}, JsonRpseeError,
    Error, error::RpcError, storage::{Address as StorageAddress}, ext::subxt_core::{utils::Yes, constants::address::Address as ConstantAddress},
    lightclient::{ChainConfig, LightClient, JsonRpcError}, config::polkadot::PolkadotExtrinsicParamsBuilder,
};
use crate::bool::runtime_types::ethereum::transaction::{EIP1559Transaction, TransactionV2 as EvmTransaction, TransactionAction};

#[derive(Clone, Debug)]
pub enum BoolConfig {}

impl Config for BoolConfig {
    type Hash = Hash;
    type AccountId = bnk_node_primitives::AccountId20;
    type Address = sp_runtime::MultiAddress<bnk_node_primitives::AccountId20, ()>;
    type Signature = bnk_node_primitives::EthereumSignature;
    type Hasher = BlakeTwo256;
    type Header = SubstrateHeader<u32, BlakeTwo256>;
    type ExtrinsicParams = PolkadotExtrinsicParams<Self>;
    type AssetId = ();
}

#[derive(Clone)]
pub struct SubClient<C: Config, P: Signer<C> + Clone> {
    pub ws_url: String,
    pub signer: Option<P>,
    pub client: Arc<RwLock<OnlineClient<C>>>,
    pub inner_nonce: Arc<RwLock<u64>>,
    // number of cache, will re-submit call if 'call_cache' length up to it.
    pub cache_size_for_call: u32,
    // call cache with target nonce, ture value for 'bool' param means the tx is submitted by evm, 'Vec<u8>' is input for evm tx, 'u128' is tx tip for priority.
    pub call_cache: Arc<RwLock<HashMap<u64, (Box<dyn TxPayload + Send + Sync>, bool, Vec<u8>, u128)>>>,
    // milliseconds, default 10000 milllis(10 seconds)
    pub warn_time: u128,
}

impl SubClient<BoolConfig, BoolSigner<BoolConfig>> {
    pub async fn new(url: &str, id: &str, password_override: Option<String>, warn_time: Option<u128>, cache_size_for_call: Option<u32>) -> SubClient<BoolConfig, BoolSigner<BoolConfig>> {
        let password_override = password_override.unwrap_or("".to_string());
        let phase = id.to_owned() + &password_override;
        let seed = sp_core::keccak_256(phase.as_bytes());
        let signer = BoolSigner::new(SecretKey::parse(&seed).expect("phase sk from seed should successfully"));
        let subxt_client = OnlineClient::<BoolConfig>::from_url(url).await.unwrap();
        let chain_nonce = subxt_client.tx().account_nonce(signer.account_id()).await.unwrap();
        SubClient {
            ws_url: url.to_string(),
            signer: Some(signer),
            client: Arc::new(RwLock::new(subxt_client)),
            inner_nonce: Arc::new(RwLock::new(chain_nonce)),
            cache_size_for_call: cache_size_for_call.unwrap_or(10),
            call_cache: Arc::new(RwLock::new(HashMap::new())),
            warn_time: warn_time.unwrap_or(10000),
        }
    }


    pub async fn new_from_ecdsa_sk(url: String, sk: Option<String>, warn_time: Option<u128>, cache_size_for_call: Option<u32>) -> Result<SubClient<BoolConfig, BoolSigner<BoolConfig>>, String> {
        let mut chain_nonce = 0;
        let subxt_client = OnlineClient::<BoolConfig>::from_insecure_url(&url).await.map_err(|e| e.to_string())?;
        let signer = if let Some(sk) = sk {
            let sk = hex::decode(sk.strip_prefix("0x").unwrap_or(&sk)).map_err(|e| e.to_string())?;
            let signer = BoolSigner::new(SecretKey::parse_slice(&sk).map_err(|e| e.to_string())?);
            chain_nonce = subxt_client.tx().account_nonce(signer.account_id()).await.unwrap();
            Some(signer)
        } else {
            None
        };
        Ok(SubClient {
            ws_url: url,
            signer,
            client: Arc::new(RwLock::new(subxt_client)),
            inner_nonce: Arc::new(RwLock::new(chain_nonce)),
            cache_size_for_call: cache_size_for_call.unwrap_or(10),
            call_cache: Arc::new(RwLock::new(HashMap::new())),
            warn_time: warn_time.unwrap_or(10000),
        })
    }

    pub async fn new_light_client(_chain_spec: &str, node_ws_url: Option<String>, sk: Option<String>, warn_time: Option<u128>, cache_size_for_call: Option<u32>) -> Result<SubClient<BoolConfig, BoolSigner<BoolConfig>>, String> {
        use subxt::utils::fetch_chainspec_from_rpc_node;
        let chain_spec = fetch_chainspec_from_rpc_node(&node_ws_url.clone().unwrap()).await.map_err(|e| e.to_string())?;
        // println!("chain_spec json: {:?}", chain_spec.get());
        // let chain_config = ChainConfig::chain_spec(chain_spec);
        let chain_config = ChainConfig::chain_spec(chain_spec.get());
        let (_light_client, chain_rpc) = LightClient::relay_chain(chain_config).map_err(|e| e.to_string())?;
        let subxt_client = OnlineClient::<BoolConfig>::from_rpc_client(chain_rpc).await.map_err(|e| e.to_string())?;
        let signer = if let Some(sk) = sk {
            let sk = hex::decode(sk.strip_prefix("0x").unwrap_or(&sk)).map_err(|e| e.to_string()).map_err(|e| e.to_string())?;
            let signer = BoolSigner::new(SecretKey::parse_slice(&sk).map_err(|e| e.to_string())?);
            Some(signer)
        } else {
            None
        };
        Ok(SubClient {
            ws_url: node_ws_url.unwrap(),
            signer,
            client: Arc::new(RwLock::new(subxt_client)),
            inner_nonce: Arc::new(RwLock::new(0)),
            cache_size_for_call: cache_size_for_call.unwrap_or(10),
            call_cache: Arc::new(RwLock::new(HashMap::new())),
            warn_time: warn_time.unwrap_or(10000),
        })
    }

    pub async fn submit_extrinsic_with_signer_and_watch<
        Call: TxPayload + 'static + Send + Sync,
    >(
        &self,
        call: Call,
        nonce: Option<u32>,
    ) -> Result<Hash, Error> {
        let call = Box::new(call);
        let timer =   Instant::now();
        self.check_client_runtime_version_and_update().await?;

        let mut inner_nonce = self.inner_nonce.write().await;
        let mut call_cache = self.call_cache.write().await;
        let client = self.client.read().await;
        let signer = self.signer.as_ref().ok_or_else(|| Error::Other("empty sk to sign and submit tx".to_string()))?;
        let account_id = signer.account_id();

        let target_nonce = if let Some(nonce) = nonce {
            nonce  as u64
        } else {
            let chain_nonce = client.tx().account_nonce(account_id).await? as u64;
            // clear cache for lower nonce, retain 10 nonce due to 'chain_nonce' can roll back
            let oldest_nonce = std::cmp::max(chain_nonce, 10);
            let old_nonce = call_cache.keys().filter(|v| v < &&(oldest_nonce - 10)).cloned().collect::<Vec<_>>();
            for key in old_nonce {
                log::trace!(target: "subxt::call_cache", "remove key {:?}", key);
                call_cache.remove(&key);
            }
            if chain_nonce >= *inner_nonce {
                chain_nonce
            } else {
                // Some errors occurred. ie. some tx with nonce not submit to chain seccessfully.
                if *inner_nonce - chain_nonce > self.cache_size_for_call as u64 {
                    log::warn!(target: "subxt", "Some errors occurred to nonce inner {}, chain {}", *inner_nonce, chain_nonce);
                    for key in chain_nonce..*inner_nonce {
                        if let Some((inner_call, by_evm, _, _)) = call_cache.get(&key) {
                            let tx = if *by_evm {
                                client.tx().create_unsigned(inner_call)?
                            } else {
                                client.tx().create_signed(
                                    inner_call,
                                    signer,
                                    PolkadotExtrinsicParamsBuilder::new().nonce(key).build(),
                                ).await?
                            };
                            let tx_hash = tx
                                .submit()
                                .await;
                            log::warn!(target: "subxt", "re-submit call with nonce: {}, res: {:?}", key, tx_hash);
                        } else {
                            log::warn!(target: "subxt", "re-submit call not find nonce: {} in cache", key);
                        }
                    }
                }
                *inner_nonce
            }
        };
        let tx: subxt::tx::SubmittableExtrinsic<BoolConfig, OnlineClient<BoolConfig>> = client.tx().create_signed(
            &call,
            signer,
            PolkadotExtrinsicParamsBuilder::new().nonce(target_nonce).build(),
        ).await?;
        let tx_hash = match tx
            .submit_and_watch()
            .await?
            .wait_for_finalized()
            .await
        {
            Ok(tx) => {
                log::debug!(target: "subxt::nonce", "inner_nonce {}, insert cache for nonce: {}", target_nonce + 1, target_nonce);
                *inner_nonce = target_nonce + 1;
                // update call_cache
                call_cache.insert(target_nonce, (call, false, vec![], 0));
                tx.wait_for_success().await?.extrinsic_hash()
            },
            Err(e) => return Err(e)
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "submit_extrinsic_with_signer_and_watch exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(tx_hash)
    }

    pub async fn submit_extrinsic_with_signer_without_watch<
        Call: TxPayload + 'static + Send + Sync,
    >(
        &self,
        call: Call,
        nonce: Option<u32>,
    ) -> Result<Hash, Error> {
        let call = Box::new(call);
        let timer = Instant::now();
        self.check_client_runtime_version_and_update().await?;

        let mut inner_nonce = self.inner_nonce.write().await;
        let mut call_cache = self.call_cache.write().await;
        let client = self.client.read().await;
        let signer = self.signer.as_ref().ok_or_else(|| Error::Other("empty sk to sign and submit tx".to_string()))?;
        let account_id = signer.account_id();

        let target_nonce = if let Some(nonce) = nonce {
            nonce as u64
        } else {
            let chain_nonce = client.tx().account_nonce(account_id).await? as u64;
            // clear cache for lower nonce
            let old_nonce = call_cache.keys().filter(|v| v < &&chain_nonce).cloned().collect::<Vec<_>>();
            for key in old_nonce {
                log::trace!(target: "subxt::call_cache", "remove key {:?}", key);
                call_cache.remove(&key);
            }
            if chain_nonce >= *inner_nonce {
                chain_nonce
            } else {
                // Some errors occurred. ie. some tx with nonce not submit to chain seccessfully.
                if *inner_nonce - chain_nonce >= self.cache_size_for_call as u64 {
                    log::warn!(target: "subxt", "Some errors occurred to nonce inner {}, chain {}", *inner_nonce, chain_nonce);
                    for key in chain_nonce ..*inner_nonce {
                        if let Some((inner_call, by_evm, input, tip)) = call_cache.get_mut(&key) {
                            let tx = if *by_evm {
                                let mut eip1995_tx = <ethereum::EIP1559Transaction as codec::Decode>::decode(&mut input.as_slice())?;
                                eip1995_tx.max_priority_fee_per_gas = eip1995_tx.max_priority_fee_per_gas + sp_core::U256::from(*tip + 100u128);
                                let evm_tx = self.build_eip1559_tx_to_v2(eip1995_tx).map_err(|e| Error::Other(e))?;
                                let evm_call = crate::bool::tx().ethereum().transact(evm_tx);
                                client.tx().create_unsigned(&evm_call)?
                            } else {
                                client.tx().create_signed(
                                    inner_call,
                                    signer,
                                    PolkadotExtrinsicParamsBuilder::new().nonce(key).tip(*tip + 100).build(),
                                ).await?
                            };
                            let tx_hash = tx
                                .submit()
                                .await;
                            log::warn!(target: "subxt", "re-submit call with nonce: {}, tip: {:?}, res: {:?}", key, *tip + 100, tx_hash);
                            //update tip
                            *tip += 100;
                        } else {
                            log::warn!(target: "subxt", "re-submit call not find nonce: {} in cache", key);
                        }
                    }
                }
                *inner_nonce
            }
        };
        let tx = client.tx().create_signed(
            &call,
            signer,
            PolkadotExtrinsicParamsBuilder::new().nonce(target_nonce).build(),
        ).await?;
        let tx_hash = match tx
            .submit()
            .await
        {
            Ok(tx) => {
                log::debug!(target: "subxt::nonce", "inner_nonce {}, insert cache for nonce: {}", target_nonce + 1, target_nonce);
                *inner_nonce = target_nonce + 1;
                // update call_cache
                call_cache.insert(target_nonce, (call, false, vec![], 0));
                tx
            },
            Err(e) => return Err(e)
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "submit_extrinsic_with_signer_and_watch exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(tx_hash)
    }

    pub async fn submit_extrinsic_without_signer<Call: TxPayload + 'static + Send + Sync>(
        &self,
        call: Call,
    ) -> Result<Hash, Error> {
        let timer = Instant::now();
        let client = self.client.read().await;
        let tx = client.tx().create_unsigned(&Box::new(call))?;
        let tx_hash = tx.submit().await?;
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "submit_extrinsic_without_signer exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(tx_hash)
    }

    pub async fn submit_extrinsic_without_signer_and_watch<Call: TxPayload>(
        &self,
        call: Call,
    ) -> Result<TxProgress<BoolConfig, OnlineClient<BoolConfig>>, Error> {
        let timer = Instant::now();
        let client = self.client.read().await;
        let tx = client.tx().create_unsigned(&call)?;
        let tx_process = tx
            .submit_and_watch()
            .await;
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "submit_extrinsic_without_signer_and_watch exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        tx_process
    }

    pub async fn query_storage<F: StorageAddress<IsFetchable = Yes> + 'static + Sized>(
        &self,
        store_query: F,
        at_block: Option<Hash>,
    ) -> Result<Option<F::Target>, Error> {
        let timer =   Instant::now();
        self.check_client_runtime_version_and_update().await?;
        let storage_client = self.client.read().await.storage();
        let res = match at_block {
            Some(block) => {
                storage_client.at(block).fetch(&store_query).await
            },
            None => {
                storage_client.at_latest().await?.fetch(&store_query).await
            }
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_storage exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn query_storage_value_iter<F: StorageAddress<IsIterable = Yes> + 'static + Sized>(
        &self,
        store_query: F,
        at_block: Option<Hash>,
    ) -> Result<Vec<(Vec<u8>, F::Target)>, Error> {
        let timer = Instant::now();
        self.check_client_runtime_version_and_update().await?;
        let storage_client = self.client.read().await.storage();
        let mut iter = match at_block {
            Some(block) => {
                storage_client.at(block).iter(store_query).await?
            },
            None => {
                storage_client.at_latest().await?.iter(store_query).await?
            }
        };
        let mut values = Vec::new();
        while let Some(Ok(kv)) = iter.next().await {
            values.push((kv.key_bytes.clone(), kv.value))
        }
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_storage exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(values)
    }

    pub async fn query_storage_or_default<F: StorageAddress<IsFetchable = Yes, IsDefaultable = Yes> + 'static + Sized>(
        &self,
        store_query: F,
        at_block: Option<Hash>,
    ) -> Result<F::Target, Error> {
        let timer =   Instant::now();
        self.check_client_runtime_version_and_update().await?;
        let storage_client = self.client.read().await.storage();
        let res = match at_block {
            Some(block) => {
                storage_client.at(block).fetch_or_default(&store_query).await
            },
            None => {
                storage_client.at_latest().await?.fetch_or_default(&store_query).await
            }
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_storage_or_default exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn query_constant<Address: ConstantAddress>(
        &self,
        address: Address,
    ) -> Result<Address::Target, Error> {
        let timer =   Instant::now();
        self.check_client_runtime_version_and_update().await?;
        let client = self.client.read().await.constants();
        let res = client.at(&address);
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_constant exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn query_account_nonce(&self) -> Option<u64> {
        let timer =   Instant::now();
        self.check_client_runtime_version_and_update().await.ok()?;
        let res = match self.client.read().await.tx().account_nonce(&self.account_id().await).await {
            Ok(nonce) => Some(nonce),
            Err(_) => None,
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_account_nonce exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn account_id(&self) -> AccountId20 {
        let timer =   Instant::now();
        let res = self.signer.as_ref()
            .expect("Bool subclient should has account")
            .account_id()
            .clone();
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "account_id exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub fn build_eip1559_tx_to_v2(&self, tx: ethereum::EIP1559Transaction) -> Result<EvmTransaction, String> {
        let tx = ethereum::EIP1559TransactionMessage::from(tx);
        let sk = self.signer.clone().ok_or("Not set bool client signer")?.signer().serialize();
        let secret = secp256k1::SecretKey::parse(&sk).map_err(|e| format!("Parse bool signer sk failed for: {:?}", e))?;
        let signing_message = secp256k1::Message::parse_slice(&tx.hash()[..])
            .map_err(|e| e.to_string())?;
        let (signature, recid) = secp256k1::sign(&signing_message, &secret);
        let rs = signature.serialize();
        let r = Hash::from_slice(&rs[0..32]);
        let s = Hash::from_slice(&rs[32..64]);
        Ok(
            EvmTransaction::EIP1559(EIP1559Transaction {
                chain_id: tx.chain_id,
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
                action: match tx.action {
                    ethereum::TransactionAction::Call(addr) => TransactionAction::Call(addr),
                    _ => return Err(format!("Invalid evm tx action: {:?}", tx.action))
                },
                value: crate::bool::runtime_types::primitive_types::U256(
                    tx.value.0
                ),
                input: tx.input,
                access_list: vec![],
                odd_y_parity: recid.serialize() != 0,
                r,
                s,
            })
        )
    }
}


impl<C: Config, P: Signer<C> + Clone> SubClient<C, P> {
    pub async fn new_from_signer(url: &str, signer: Option<P>, warn_time: Option<u128>, cache_size_for_call: Option<u32>) -> Result<SubClient<C, P>, Error> {
        let ws_url: url::Url = url.parse().map_err(|_| Error::Other("parse url from string failed".to_string()))?;
        let mut fixed_ws_url = ws_url.as_str().to_string();
        if ws_url.port().is_none() {
            let mut tmp = vec![fixed_ws_url.strip_suffix(ws_url.path()).unwrap_or(&fixed_ws_url)];
            let default_port = format!(":{}", default_port(ws_url.scheme()).unwrap());
            tmp.push(&default_port);
            tmp.push(ws_url.path());
            fixed_ws_url = tmp.concat();
        }
        let subxt_client = OnlineClient::<C>::from_url(fixed_ws_url.clone()).await?;
        Ok(
            SubClient {
                ws_url: fixed_ws_url,
                signer,
                client: Arc::new(RwLock::new(subxt_client)),
                inner_nonce: Arc::new(RwLock::new(0)),
                cache_size_for_call: cache_size_for_call.unwrap_or(10),
                call_cache: Arc::new(RwLock::new(HashMap::new())),
                warn_time: warn_time.unwrap_or(10000),
            }
        )
    }

    pub async fn check_client_runtime_version_and_update(&self) -> Result<(), Error> {
        let timer =   Instant::now();
        let client = self.client.read().await;
        let res = match client.backend().current_runtime_version().await {
            Ok(runtime_version) => if runtime_version != client.runtime_version() {
                log::warn!(target: "subxt", "invalid runtime version, try to rebuild client...");
                drop(client);
                self.rebuild_client().await
            } else {
                Ok(())
            },
            Err(e) => {
                log::warn!(target: "subxt", "get remote runtime version for: {:?}", e);
                drop(client);
                self.handle_error(e).await
            },
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "check_client_runtime_version_and_update exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn rebuild_client(&self) -> Result<(), Error> {
        use subxt::utils::fetch_chainspec_from_rpc_node;

        let timer = Instant::now();
        let chain_spec = fetch_chainspec_from_rpc_node(&self.ws_url).await.map_err(|e| e.to_string())?;
        // println!("chain_spec json: {:?}", chain_spec.get());
        // let chain_config = ChainConfig::chain_spec(chain_spec);
        let chain_config = ChainConfig::chain_spec(chain_spec.get());
        let (_light_client, chain_rpc) = LightClient::relay_chain(chain_config).map_err(|e| e.to_string())?;
        let client = OnlineClient::<C>::from_rpc_client(chain_rpc).await.map_err(|e| e.to_string())?;
        *self.client.write().await = client;
        log::info!(target: "subxt", "rebuild client successful");
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "rebuild_client exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(())
    }

    pub async fn handle_error(&self, err: Error) -> Result<(), Error> {
        return match err {
            Error::Rpc(RpcError::SubscriptionDropped) => {
                log::warn!(target: "subxt", "rebuild client for SubscriptionDropped");
                self.rebuild_client().await
            },
            Error::Rpc(RpcError::ClientError(client_err)) => {
                if let Some(e) = client_err.downcast_ref::<JsonRpseeError>() {
                    match *e {
                        JsonRpseeError::RestartNeeded(_) => {
                            log::warn!(target: "subxt", "rebuild client for {:?}", e);
                            self.rebuild_client().await
                        },
                        _ => Err(Error::Rpc(RpcError::ClientError(client_err))),
                    }
                } else {
                    if let Some(e) = client_err.downcast_ref::<JsonRpcError>() {
                        if e.to_string().contains("No node available for storage query") {
                            log::warn!(target: "subxt", "rebuild client for {:?}", e);
                            self.rebuild_client().await
                        } else {
                            Err(Error::Rpc(RpcError::ClientError(client_err)))
                        }
                    } else {
                        // Not handle other error type now
                        Err(Error::Rpc(RpcError::ClientError(client_err)))
                    }
                }
            },
            _ => Err(err),
        }
    }
}

pub fn default_port(scheme: &str) -> Option<u16> {
    match scheme {
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        "ftp" => Some(21),
        _ => None,
    }
}

#[tokio::test]
async fn test_rebuild_client() {
    let url = "ws://127.0.0.1:9944".to_string();
    let sk = "5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133".to_string();
    let client = SubClient::new_from_ecdsa_sk(url, Some(sk), None, None).await.unwrap();
    loop {
        println!("try to query challenges");
        let res = crate::query::mining::challenges(&client, 1, None).await.unwrap();
        println!("query challenges result: {:?}", res);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}


#[tokio::test]
async fn test_query_iter() {
    let url = "wss://test-rpc-node-ws.bool.network".to_string();
    let client = crate::client::SubClient::new_from_signer(&url, None, None, None).await.unwrap();
    let res = crate::query::committee::committees_iter(&client, None).await.unwrap();
    println!("res: {res:?}");
}

#[tokio::test]
async fn test_query_cmt() {
    let url = "wss://test-rpc-node-ws.bool.network".to_string();
    let client = crate::client::SubClient::new_from_signer(&url, None, None, None).await.unwrap();

    for i in 1u32..426 {
        let res = crate::query::committee::committees(&client, i, None).await.unwrap();
        println!("res: {res:?}");
    }
}

#[tokio::test]
async fn test_query_btc_committee_type_iter() {
    let url = "ws://127.0.0.1:9933".to_string();
    let client = crate::client::SubClient::new_from_signer(&url, None, None, None).await.unwrap();
    let res = crate::query::channel::btc_committee_type_iter(&client, None).await.unwrap();
    println!("res: {res:?}");
}

#[tokio::test]
async fn test_nonce_roll_back() {
    use subxt::ext::subxt_core::utils::AccountId20;

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    use std::str::FromStr;
    use crate::BoolSubClient;

    let url = "ws://127.0.0.1:9933".to_string();
    let sk_bytes = hex::decode("").unwrap();
    let sk = SecretKey::parse_slice(&sk_bytes).unwrap();
    let signer = BoolSigner::new(sk);
    let client = BoolSubClient::new_from_signer(&url, Some(signer), None, Some(20)).await.unwrap();
    let account = AccountId20::from_str("0x89Bdaf4AC10bC9d497BCa9a5cc37972026146E0E").unwrap();
    let dst = AccountId20(account.0);

    for i in 0..200 {
        log::info!("index: {i}");
        let call = crate::bool::tx().balances().transfer_keep_alive(dst.into(), 100000);
        let res = client.submit_extrinsic_with_signer_without_watch(call, None).await.map_err(|e| e.to_string());
        log::info!("submit res: {res:?}");
    }
}

#[tokio::test]
async fn test_light_client() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // let chain_spec = include_str!("../../node/res/local.json");
    let light_client = crate::client::SubClient::new_light_client(
        "", Some("ws://127.0.0.1:9933".to_string()), None, None, None
    ).await.unwrap();
    println!("light_client build");

    let storage_client = light_client.client.read().await.storage();

    loop {
        let storage_query1 = crate::bool::storage().system().number();
        let latest_height = storage_client.at_latest().await.unwrap().fetch(&storage_query1).await.unwrap().unwrap() - 1;


        let storage_query2 = crate::bool::storage().system().block_hash(latest_height);
        let latest_hash = storage_client.at_latest().await.unwrap().fetch(&storage_query2).await.unwrap().unwrap();
        println!("*************latest_height: {:?}, ****************latest_hash: {:?}", latest_height, latest_hash);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}


