#![deny(unused_crate_dependencies)]
pub mod client;
pub mod event_watcher;
pub mod monitor_rpc;
pub mod query;
pub mod submit;
pub mod types;
pub mod watcher_rpc;

pub use crate::client::BoolConfig;
use bnk_node_primitives::CustomError;
pub use bnk_node_primitives;
pub use subxt::constants::Address;
pub use subxt::events::StaticEvent;
pub use subxt::tx::{BoolSigner, SecretKey};
pub use subxt::{
    config::extrinsic_params::BaseExtrinsicParamsBuilder, error::RpcError, events::EventDetails,
    subxt, Error, JsonRpseeError,
};

/// use subxt cli to update metadata 'subxt metadata --url http://127.0.0.1:9933 --version 14 -f bytes > metadata.scale'
#[subxt::subxt(
    runtime_metadata_path = "./metadata.scale",
    derive_for_all_types = "Eq, PartialEq, Clone, Debug"
)]
pub mod bool {}

pub type BoolSubClient = client::SubClient<BoolConfig, BoolSigner<BoolConfig>>;

#[derive(Debug, PartialEq)]
pub enum CommitteeEvent {
    CreateCommittee,
    CommitteeCreateFinished,
    ApplyEpochChange,
    BindAnchor,
    CommitteeStartWork,
    StopCommittee,
    RefreshAssets,
    UpdateConfigs,
    UpdatePoolRate,
    KeyGenerate,
    KeyHandover,
    ExposeIdentity,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum CommitteeHealthEvent {
    Challenges,
    HealthReport,
    ConfirmDHCState,
    PunishEvilDevice,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum ConfigsEvent {
    SetRoundMsgWait,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum ChannelEvent {
    NewTransaction,
    SubmitTransaction,
    Connection,
    NewSourceHash,
    CreateNewTx,
    RefreshInscription,
    SignRefresh,
    SubmitRefresh,
    RequestNewIssueXudt,
    SignIssueXudt,
    SignIssueXudtFinished,
    UpdateIssueXudtStatus,
    SignNewUid,
    SubmitSignNewUidResult,
    UpdateChannelMappingTick,
    UpdateCommitteeFeeConfig,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum MinningEvent {
    NewChallenge,
    Heartbeat,
    DeviceJoinService,
    DeviceTryExitService,
    DeviceExitService,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum OracleEvent {
    TriggerSign,
    NewRandomNumber,
    NewBrc20IndexData,
    SubmitBrc20ConsensusResult,
    ReEmitBrc20ConsensusResult,
    Brc20OracleRequest,
    Brc20OracleSignResult,
    Unknown,
}

impl std::str::FromStr for CommitteeEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<CommitteeEvent, Self::Err> {
        match input {
            "CreateCommittee" => Ok(CommitteeEvent::CreateCommittee),
            "CommitteeCreateFinished" => Ok(CommitteeEvent::CommitteeCreateFinished),
            "ApplyEpochChange" => Ok(CommitteeEvent::ApplyEpochChange),
            "BindAnchor" => Ok(CommitteeEvent::BindAnchor),
            "CommitteeStartWork" => Ok(CommitteeEvent::CommitteeStartWork),
            "StopCommittee" => Ok(CommitteeEvent::StopCommittee),
            "RefreshAssets" => Ok(CommitteeEvent::RefreshAssets),
            "UpdateConfigs" => Ok(CommitteeEvent::UpdateConfigs),
            "UpdatePoolRate" => Ok(CommitteeEvent::UpdatePoolRate),
            "KeyGenerate" => Ok(CommitteeEvent::KeyGenerate),
            "KeyHandover" => Ok(CommitteeEvent::KeyHandover),
            "ExposeIdentity" => Ok(CommitteeEvent::ExposeIdentity),
            _ => Ok(CommitteeEvent::Unknown),
        }
    }
}

impl std::str::FromStr for CommitteeHealthEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<CommitteeHealthEvent, Self::Err> {
        match input {
            "Challenges" => Ok(CommitteeHealthEvent::Challenges),
            "HealthReport" => Ok(CommitteeHealthEvent::HealthReport),
            "ConfirmDHCState" => Ok(CommitteeHealthEvent::ConfirmDHCState),
            "PunishEvilDevice" => Ok(CommitteeHealthEvent::PunishEvilDevice),
            _ => Ok(CommitteeHealthEvent::Unknown),
        }
    }
}

impl std::str::FromStr for ConfigsEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<ConfigsEvent, Self::Err> {
        match input {
            "SetRoundMsgWait" => Ok(ConfigsEvent::SetRoundMsgWait),
            _ => Ok(ConfigsEvent::Unknown),
        }
    }
}

impl std::str::FromStr for ChannelEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<ChannelEvent, Self::Err> {
        match input {
            "NewTransaction" => Ok(ChannelEvent::NewTransaction),
            "SubmitTransaction" => Ok(ChannelEvent::SubmitTransaction),
            "Connection" => Ok(ChannelEvent::Connection),
            "NewSourceHash" => Ok(ChannelEvent::NewSourceHash),
            "CreateNewTx" => Ok(ChannelEvent::CreateNewTx),
            "RefreshInscription" => Ok(ChannelEvent::RefreshInscription),
            "SignRefresh" => Ok(ChannelEvent::SignRefresh),
            "SubmitRefresh" => Ok(ChannelEvent::SubmitRefresh),
            "RequestNewIssueXudt" => Ok(ChannelEvent::RequestNewIssueXudt),
            "SignIssueXudt" => Ok(ChannelEvent::SignIssueXudt),
            "SignIssueXudtFinished" => Ok(ChannelEvent::SignIssueXudtFinished),
            "UpdateIssueXudtStatus" => Ok(ChannelEvent::UpdateIssueXudtStatus),
            "UpdateChannelMappingTick" => Ok(ChannelEvent::UpdateChannelMappingTick),
            "UpdateCommitteeFeeConfig" => Ok(ChannelEvent::UpdateCommitteeFeeConfig),
            "SignNewUid" => Ok(ChannelEvent::SignNewUid),
            "SubmitSignNewUidResult" => Ok(ChannelEvent::SubmitSignNewUidResult),
            _ => Ok(ChannelEvent::Unknown),
        }
    }
}

impl std::str::FromStr for MinningEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<MinningEvent, Self::Err> {
        match input {
            "NewChallenge" => Ok(MinningEvent::NewChallenge),
            "Heartbeat" => Ok(MinningEvent::Heartbeat),
            "DeviceJoinService" => Ok(MinningEvent::DeviceJoinService),
            "DeviceTryExitService" => Ok(MinningEvent::DeviceTryExitService),
            "DeviceExitService" => Ok(MinningEvent::DeviceExitService),
            _ => Ok(MinningEvent::Unknown),
        }
    }
}

impl std::str::FromStr for OracleEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<OracleEvent, Self::Err> {
        match input {
            "TriggerSign" => Ok(OracleEvent::TriggerSign),
            "NewRandomNumber" => Ok(OracleEvent::NewRandomNumber),
            "NewBrc20IndexData" => Ok(OracleEvent::NewBrc20IndexData),
            "SubmitBrc20ConsensusResult" => Ok(OracleEvent::SubmitBrc20ConsensusResult),
            "ReEmitBrc20ConsensusResult" => Ok(OracleEvent::ReEmitBrc20ConsensusResult),
            "Brc20OracleRequest" => Ok(OracleEvent::Brc20OracleRequest),
            "Brc20OracleSignResult" => Ok(OracleEvent::Brc20OracleSignResult),
            _ => Ok(OracleEvent::Unknown),
        }
    }
}

pub(crate) fn convert_to_custom_error(custom: u8) -> String {
    let err = CustomError::from_num(custom);
    err.to_string()
}

pub(crate) fn handle_custom_error(error: Error) -> String {
    if let Error::Rpc(RpcError::ClientError(e)) = error {
        let err = e.to_string();
        parse_custom_err_from_string_err(err)
    } else {
        error.to_string()
    }
}

fn parse_custom_err_from_string_err(err: String) -> String {
    // only try to extract 'custom number', will return input if parse error
    let v: Vec<&str> = err.split("Custom error: ").collect();
    if v.len() == 2 {
        let vv: Vec<&str> = v[1].split('\"').collect();
        if vv.len() == 2 {
            if let Ok(num) = vv[0].parse::<u8>() {
                return convert_to_custom_error(num);
            }
        }
    }
    err
}

pub fn no_prefix<T: AsRef<str>>(data: T) -> String {
    data.as_ref()
        .strip_prefix("0x")
        .unwrap_or(data.as_ref())
        .to_string()
}
