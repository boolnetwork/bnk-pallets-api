//! EventWatcher for Bool node witch BoolSubClient.
use bnk_node_primitives::Hash;
use tokio::sync::mpsc::Sender;
use subxt::Config;
use subxt::events::EventDetails;
use std::cmp::Ordering;
use crate::{BoolConfig, BoolSubClient as SubClient};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum WatcherMode {
    #[default]
    Both,
    Latest,
    Finalized,
}

#[derive(Clone)]
pub struct EventWatcher {
    log_target: String,
    client: SubClient,
    handler: Sender<(WatcherMode, u32, Hash, Vec<EventDetails<BoolConfig>>)>,
    pub latest: u32,
    pub finalized: u32,
}

impl EventWatcher {
    pub fn new(
        log_target: &str,
        client: SubClient,
        handler: Sender<(WatcherMode, u32, Hash, Vec<EventDetails<BoolConfig>>)>,
    ) -> Self {
        EventWatcher {
            log_target: log_target.to_string(),
            client,
            handler,
            latest: 0,
            finalized: 0,
        }
    }

    pub async fn initialize(&mut self) {
        // initialize latest block number
        loop {
            match get_block_number(self.client.clone(), None).await {
                Ok(block_number) => {
                    self.latest = block_number;
                    break;
                }
                Err(e) => log::error!(target: &self.log_target, "initialize latest block: {e:?}"),
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        // initialize finalized block number
        loop {
            match get_block_hash(self.client.clone(), WatcherMode::Finalized).await {
                Ok(hash) => match get_block_number(self.client.clone(), Some(hash)).await {
                    Ok(block_number) => {
                        self.finalized = block_number;
                        log::info!(target: &self.log_target, "Initialize event_watcher with latest_block {}, finalized block {}", self.latest, self.finalized);
                        break;
                    }
                    Err(e) => log::error!(target: &self.log_target, "initialize finalized block: {e:?}"),
                }
                Err(e) => log::error!(target: &self.log_target, "initialize finalized block: {e:?}"),
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }

    pub fn run(mut self, mode: WatcherMode) {
        tokio::spawn(async move {
            log::info!(target: &self.log_target, "Start watching blocks by url: {}......", &self.client.ws_url);
            loop {
                if matches!(mode, WatcherMode::Latest | WatcherMode::Both) {
                    match get_block_number(self.client.clone(), None).await {
                        Ok(current_number) => {
                            match self.latest.cmp(&current_number) {
                                    Ordering::Less => {
                                    log::trace!(target: &self.log_target, "handle latest block from {:?} to {current_number}", self.latest);
                                    self.handle_blocks_events(self.latest + 1, current_number, WatcherMode::Latest).await;
                                    self.latest = current_number;
                                }
                                Ordering::Equal => log::debug!(target: &self.log_target, "caught up with the best latest block height: {current_number:?}"),
                                Ordering::Greater => log::debug!(target: &self.log_target, "latest block height is rolled back, from {:?} to {current_number:?}", self.latest),
                            }
                        },
                        Err(e) => log::error!(target: &self.log_target, "get latest block: {e:?}"),
                    };
                }

                if matches!(mode, WatcherMode::Finalized | WatcherMode::Both) {
                    match get_block_hash(self.client.clone(), WatcherMode::Finalized).await {
                        Ok(hash) => match get_block_number(self.client.clone(), Some(hash)).await {
                            Ok(current_number) => {
                                match self.finalized.cmp(&current_number) {
                                    Ordering::Less => {
                                        log::trace!(target: &self.log_target, "handle finalized block from {:?} to {current_number}", self.finalized);
                                        self.handle_blocks_events(self.finalized + 1, current_number, WatcherMode::Finalized).await;
                                        self.finalized = current_number;
                                    }
                                    Ordering::Equal => log::debug!(target: &self.log_target, "caught up with the best finalized block height: {current_number:?}"),
                                    Ordering::Greater => log::warn!(target: &self.log_target, "finalized block height is rolled back, local: {:?}, chain: {current_number:?}", self.finalized),
                                }
                            },
                            Err(e) => log::error!(target: &self.log_target, "get finalized block number err: {e:?}"),
                        },
                        Err(e) => log::error!(target: &self.log_target, "get finalized block hash err: {e:?}"),
                    };
                }

                #[cfg(feature = "telemetry")]
                {
                    bool_telemetry_client::set_best_block_number(self.latest);
                    bool_telemetry_client::set_finalized_block_number(self.finalized);
                }

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        });
    }

    /// handle blocks between [from, to]
    async fn handle_blocks_events(&self, from: u32, to: u32, mode: WatcherMode) {
        // handle block one by one
        for block in from..=to {
            match self.client.client.read().await.rpc().block_hash(Some(block.into())).await {
                Ok(hash) => {
                    match hash {
                        Some(hash) => {
                            let events = match self.client.client.read().await.events().at(hash).await {
                                Ok(events) => events,
                                Err(e) => {
                                    log::error!(target: &self.log_target, "event watcher get events by block hash: {hash:?} failed for: {e:?}");
                                    continue;
                                }
                            };
                            let events: Vec<_> = events
                                .iter()
                                .into_iter()
                                .filter_map(|event| match event {
                                    Ok(event) => Some(event),
                                    Err(e) => {
                                        log::error!(target: &self.log_target, "event decode from metadata failed for: {e:?}");
                                        None
                                    }
                                })
                                .collect();
                            if let Err(e) = self.handler.send((mode, block, hash, events)).await {
                                log::error!(target: &self.log_target, "handle_blocks_events(send events to handler err: {e:?})");
                            }
                        }
                        None => {
                            log::error!(target: &self.log_target, "handle_blocks_events(get empty block hash by number: {block:?})");
                            continue;
                        }
                    }
                }
                Err(e) => {
                    log::error!(target: &self.log_target, "handle_blocks_events(get block hash by number: {block:?} failed for: {e:?})");
                    continue;
                }
            }
        }
    }
}

pub async fn get_events(
    client: &SubClient,
    block: u32,
    pallets: Option<Vec<&str>>,
) -> anyhow::Result<(Hash, Vec<EventDetails<BoolConfig>>)> {
    let hash = client
        .client
        .read()
        .await
        .rpc()
        .block_hash(Some(block.into()))
        .await
        .map_err(|e| anyhow::anyhow!("{e:?}"))?
        .ok_or(anyhow::anyhow!("no block hash for block {block}"))?;
    let events = match client.client.read().await.events().at(hash).await {
        Ok(events) => events,
        Err(e) => anyhow::bail!("get events for block: {block}, hash: {hash:?} failed for: {e:?}"),
    };
    let mut event_details = vec![];
    for event in events.iter().into_iter() {
        match event {
            Ok(event) => {
                if let Some(pallets) = &pallets {
                    if pallets.contains(&event.pallet_name()) {
                        event_details.push(event);
                    }
                } else {
                    event_details.push(event);
                }
            },
            Err(e) => anyhow::bail!("parse event failed for {e:?}"),
        }
    }
    Ok((hash, event_details))
}

pub async fn get_block_hash(client: SubClient, mode: WatcherMode) -> Result<<BoolConfig as Config>::Hash, String> {
    let guard_client = client.client.read().await;
    match mode {
        WatcherMode::Latest => {
            match guard_client.rpc().block_hash(None).await {
                Ok(Some(hash)) => return Ok(hash),
                Ok(None) => return Err("get empty lastet block".to_string()),
                Err(e) => {
                    drop(guard_client);
                    log::error!("get latest block failed for : {e:?}, try to rebuild client");
                    let err_str = e.to_string();
                    if let Err(e) = client.handle_error(e).await {
                        return Err(e.to_string());
                    }
                    return Err(err_str);
                }
            }
        },
        WatcherMode::Finalized => {
            match guard_client.rpc().finalized_head().await {
                Ok(hash) => return Ok(hash),
                Err(e) => {
                    drop(guard_client);
                    log::error!("event watcher get finalized block failed for : {e:?}, try to rebuild client");
                    let err_str = e.to_string();
                    if let Err(e) = client.handle_error(e).await {
                        return Err(e.to_string());
                    }
                    return Err(err_str);
                }
            }
        },
        WatcherMode::Both => Err("function get_block_hash doesn't support mode: WatcherMode::Both".to_string()),
    }
}

pub async fn get_block_number(client: SubClient, hash: Option<<BoolConfig as Config>::Hash>) -> Result<u32, String> {
    let guard_client = client.client.read().await;
    match guard_client.rpc().header(hash).await {
        Ok(Some(header)) => return Ok(header.number),
        Ok(None) => return Err(format!("subxt client get empty block by hash: {hash:?}")),
        Err(e) => {
            drop(guard_client);
            log::error!("event watcher get block by hash: {hash:?} failed for: {e:?}, try to rebuild client");
            let err_str = e.to_string();
            if let Err(e) = client.handle_error(e).await {
                return Err(e.to_string());
            }
            return Err(err_str);
        },
    }
}
