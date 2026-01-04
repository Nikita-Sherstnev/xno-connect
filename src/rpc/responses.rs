//! RPC response types.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use serde::Deserialize;

use crate::types::{Account, BlockHash, Raw, Signature, Work};

/// Account balance response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountBalanceResponse {
    /// Current confirmed balance.
    pub balance: Raw,
    /// Balance including unconfirmed blocks.
    pub pending: Raw,
    /// Receivable balance (newer term for pending).
    #[serde(default)]
    pub receivable: Option<Raw>,
}

/// Account info response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountInfoResponse {
    /// Account frontier (latest block hash).
    pub frontier: BlockHash,
    /// Open block hash.
    pub open_block: BlockHash,
    /// Representative block hash.
    pub representative_block: BlockHash,
    /// Current balance.
    pub balance: Raw,
    /// Last modified timestamp.
    pub modified_timestamp: String,
    /// Block count.
    pub block_count: String,
    /// Account version.
    #[serde(default)]
    pub account_version: Option<String>,
    /// Representative account.
    #[serde(default)]
    pub representative: Option<Account>,
    /// Voting weight.
    #[serde(default)]
    pub weight: Option<Raw>,
    /// Pending/receivable balance.
    #[serde(default)]
    pub pending: Option<Raw>,
    /// Receivable balance.
    #[serde(default)]
    pub receivable: Option<Raw>,
    /// Confirmation height.
    #[serde(default)]
    pub confirmation_height: Option<String>,
    /// Confirmation height frontier.
    #[serde(default)]
    pub confirmation_height_frontier: Option<BlockHash>,
}

/// Account history entry.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountHistoryEntry {
    /// Block type.
    #[serde(rename = "type")]
    pub block_type: String,
    /// Account involved.
    pub account: Account,
    /// Amount transferred.
    pub amount: Raw,
    /// Local timestamp.
    pub local_timestamp: String,
    /// Block height.
    pub height: String,
    /// Block hash.
    pub hash: BlockHash,
}

/// Account history response.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountHistoryResponse {
    /// Account address.
    pub account: Account,
    /// Transaction history.
    pub history: Vec<AccountHistoryEntry>,
    /// Previous block hash for pagination.
    #[serde(default)]
    pub previous: Option<BlockHash>,
}

/// Receivable blocks for an account.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountsReceivableResponse {
    /// Map of account -> list of block hashes or block info.
    pub blocks: BTreeMap<String, serde_json::Value>,
}

/// Block info response.
#[derive(Debug, Clone, Deserialize)]
pub struct BlockInfoResponse {
    /// Block account.
    pub block_account: Account,
    /// Amount transferred.
    pub amount: Raw,
    /// Balance after block.
    pub balance: String,
    /// Block height.
    pub height: String,
    /// Local timestamp.
    pub local_timestamp: String,
    /// Whether confirmed.
    pub confirmed: String,
    /// Block contents.
    pub contents: BlockContents,
    /// Block subtype.
    #[serde(default)]
    pub subtype: Option<String>,
}

/// Block contents within block info.
#[derive(Debug, Clone, Deserialize)]
pub struct BlockContents {
    /// Block type (always "state" for state blocks).
    #[serde(rename = "type")]
    pub block_type: String,
    /// Account.
    pub account: Option<Account>,
    /// Previous block hash.
    pub previous: Option<BlockHash>, // Could be genesis block
    /// Representative.
    pub representative: Option<Account>,
    /// Balance.
    pub balance: Option<String>,
    /// Link field.
    pub link: Option<String>,
    /// Link as account (for sends).
    #[serde(default)]
    pub link_as_account: Option<Account>,
    /// Signature.
    pub signature: Signature,
    /// Work.
    pub work: Work,
}

/// Block count response.
#[derive(Debug, Clone, Deserialize)]
pub struct BlockCountResponse {
    /// Total blocks.
    pub count: String,
    /// Unchecked blocks.
    pub unchecked: String,
    /// Cemented blocks.
    #[serde(default)]
    pub cemented: Option<String>,
}

/// Process block response.
#[derive(Debug, Clone, Deserialize)]
pub struct ProcessResponse {
    /// Hash of the processed block.
    pub hash: BlockHash,
}

/// Work generate response.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkGenerateResponse {
    /// Generated work.
    pub work: Work,
    /// Difficulty achieved.
    #[serde(default)]
    pub difficulty: Option<String>,
    /// Multiplier achieved.
    #[serde(default)]
    pub multiplier: Option<String>,
    /// Hash used.
    #[serde(default)]
    pub hash: Option<BlockHash>,
}

/// Version response.
#[derive(Debug, Clone, Deserialize)]
pub struct VersionResponse {
    /// RPC version.
    pub rpc_version: String,
    /// Store version.
    pub store_version: String,
    /// Protocol version.
    pub protocol_version: String,
    /// Node vendor.
    pub node_vendor: String,
    /// Store vendor.
    #[serde(default)]
    pub store_vendor: Option<String>,
    /// Network.
    #[serde(default)]
    pub network: Option<String>,
    /// Network identifier.
    #[serde(default)]
    pub network_identifier: Option<String>,
    /// Build info.
    #[serde(default)]
    pub build_info: Option<String>,
}

/// Peers response.
#[derive(Debug, Clone, Deserialize)]
pub struct PeersResponse {
    /// Map of peer address -> protocol version.
    pub peers: BTreeMap<String, String>,
}

/// Telemetry response.
#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryResponse {
    /// Block count.
    pub block_count: String,
    /// Cemented count.
    pub cemented_count: String,
    /// Unchecked count.
    pub unchecked_count: String,
    /// Account count.
    pub account_count: String,
    /// Bandwidth cap.
    pub bandwidth_cap: String,
    /// Peer count.
    pub peer_count: String,
    /// Protocol version.
    pub protocol_version: String,
    /// Uptime.
    pub uptime: String,
    /// Genesis block.
    pub genesis_block: BlockHash,
    /// Major version.
    pub major_version: String,
    /// Minor version.
    pub minor_version: String,
    /// Patch version.
    pub patch_version: String,
    /// Pre-release version.
    pub pre_release_version: String,
    /// Maker.
    pub maker: String,
    /// Timestamp.
    pub timestamp: String,
    /// Active difficulty.
    #[serde(default)]
    pub active_difficulty: Option<String>,
}

/// Representatives response.
#[derive(Debug, Clone, Deserialize)]
pub struct RepresentativesResponse {
    /// Map of representative account -> voting weight.
    pub representatives: BTreeMap<String, Raw>,
}

/// Representatives online response.
#[derive(Debug, Clone, Deserialize)]
pub struct RepresentativesOnlineResponse {
    /// List or map of online representatives.
    pub representatives: serde_json::Value,
}

/// Available supply response.
#[derive(Debug, Clone, Deserialize)]
pub struct AvailableSupplyResponse {
    /// Available supply in raw.
    pub available: Raw,
}

/// Frontier count response.
#[derive(Debug, Clone, Deserialize)]
pub struct FrontierCountResponse {
    /// Number of accounts.
    pub count: String,
}

/// Confirmation quorum response.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfirmationQuorumResponse {
    /// Quorum delta.
    pub quorum_delta: Raw,
    /// Online weight quorum percent.
    pub online_weight_quorum_percent: String,
    /// Online weight minimum.
    pub online_weight_minimum: Raw,
    /// Online stake total.
    pub online_stake_total: Raw,
    /// Trended stake total.
    #[serde(default)]
    pub trended_stake_total: Option<Raw>,
    /// Peers stake total.
    pub peers_stake_total: Raw,
}

/// Generic error response.
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    /// Error message.
    pub error: String,
}

/// Check if a response contains an error.
pub fn check_error(json: &serde_json::Value) -> Option<String> {
    json.get("error").and_then(|e| e.as_str()).map(String::from)
}
