//! RPC request builders.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::Serialize;

use crate::types::{Account, BlockHash, StateBlock, Work};

/// RPC action for account_balance.
#[derive(Debug, Serialize)]
pub struct AccountBalanceRequest {
    /// The RPC action name.
    pub action: String,
    /// The account address to query.
    pub account: String,
}

impl AccountBalanceRequest {
    /// Create a new account_balance request.
    pub fn new(account: &Account) -> Self {
        AccountBalanceRequest {
            action: "account_balance".to_string(),
            account: account.as_str().to_string(),
        }
    }
}

/// RPC action for account_info.
#[derive(Debug, Serialize)]
pub struct AccountInfoRequest {
    /// The RPC action name.
    pub action: String,
    /// The account address to query.
    pub account: String,
    /// Include representative in response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub representative: Option<bool>,
    /// Include voting weight in response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<bool>,
    /// Include pending balance (deprecated, use receivable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending: Option<bool>,
    /// Include receivable balance in response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receivable: Option<bool>,
}

impl AccountInfoRequest {
    /// Create a new account_info request with default options.
    pub fn new(account: &Account) -> Self {
        AccountInfoRequest {
            action: "account_info".to_string(),
            account: account.as_str().to_string(),
            representative: Some(true),
            weight: Some(true),
            pending: None,
            receivable: Some(true),
        }
    }
}

/// RPC action for account_history.
#[derive(Debug, Serialize)]
pub struct AccountHistoryRequest {
    /// The RPC action name.
    pub action: String,
    /// The account address to query.
    pub account: String,
    /// Maximum number of history entries to return.
    pub count: String,
    /// Optional block hash to start from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    /// Optional offset for pagination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Return results in reverse chronological order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse: Option<bool>,
}

impl AccountHistoryRequest {
    /// Create a new account_history request.
    pub fn new(account: &Account, count: u64) -> Self {
        AccountHistoryRequest {
            action: "account_history".to_string(),
            account: account.as_str().to_string(),
            count: count.to_string(),
            head: None,
            offset: None,
            reverse: None,
        }
    }

    /// Set the starting block hash for pagination.
    pub fn with_head(mut self, head: &BlockHash) -> Self {
        self.head = Some(head.to_hex());
        self
    }

    /// Set the offset for pagination.
    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Return results in reverse chronological order.
    pub fn reversed(mut self) -> Self {
        self.reverse = Some(true);
        self
    }
}

/// RPC action for accounts_receivable.
#[derive(Debug, Serialize)]
pub struct AccountsReceivableRequest {
    /// The RPC action name.
    pub action: String,
    /// List of account addresses to query.
    pub accounts: Vec<String>,
    /// Maximum number of receivable blocks per account.
    pub count: String,
    /// Minimum amount threshold in raw.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<String>,
    /// Include source account in response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<bool>,
}

impl AccountsReceivableRequest {
    /// Create a new accounts_receivable request.
    pub fn new(accounts: &[Account], count: u64) -> Self {
        AccountsReceivableRequest {
            action: "accounts_receivable".to_string(),
            accounts: accounts.iter().map(|a| a.as_str().to_string()).collect(),
            count: count.to_string(),
            threshold: None,
            source: Some(true),
        }
    }

    /// Set minimum amount threshold in raw.
    pub fn with_threshold(mut self, threshold_raw: &str) -> Self {
        self.threshold = Some(threshold_raw.to_string());
        self
    }
}

/// RPC action for block_info.
#[derive(Debug, Serialize)]
pub struct BlockInfoRequest {
    /// The RPC action name.
    pub action: String,
    /// The block hash to query.
    pub hash: String,
    /// Return block contents as JSON object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_block: Option<bool>,
}

impl BlockInfoRequest {
    /// Create a new block_info request.
    pub fn new(hash: &BlockHash) -> Self {
        BlockInfoRequest {
            action: "block_info".to_string(),
            hash: hash.to_hex(),
            json_block: Some(true),
        }
    }
}

/// RPC action for block_count.
#[derive(Debug, Serialize)]
pub struct BlockCountRequest {
    /// The RPC action name.
    pub action: String,
}

impl BlockCountRequest {
    /// Create a new block_count request.
    pub fn new() -> Self {
        BlockCountRequest {
            action: "block_count".to_string(),
        }
    }
}

impl Default for BlockCountRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// RPC action for process (submit block).
#[derive(Debug, Serialize)]
pub struct ProcessRequest {
    /// The RPC action name.
    pub action: String,
    /// Indicates block is in JSON format (must be "true" string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_block: Option<String>,
    /// Block subtype (send, receive, open, change, epoch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    /// The block to process.
    pub block: ProcessBlock,
}

/// Block format for process request (includes link_as_account).
#[derive(Debug, Serialize)]
pub struct ProcessBlock {
    /// Block type (always "state" for state blocks).
    #[serde(rename = "type")]
    pub block_type: String,
    /// The account this block belongs to.
    pub account: String,
    /// Hash of the previous block (zero for open blocks).
    pub previous: String,
    /// The representative for this account.
    pub representative: String,
    /// The balance after this block in raw.
    pub balance: String,
    /// The link field (destination/source depending on subtype).
    pub link: String,
    /// The link field interpreted as an account address.
    pub link_as_account: String,
    /// The block signature.
    pub signature: String,
    /// The proof of work.
    pub work: String,
}

impl ProcessRequest {
    /// Create a new process request from a state block.
    pub fn new(block: StateBlock) -> Self {
        let subtype = block.subtype.as_ref().map(|s| s.as_str().to_string());

        // Convert link to account format for link_as_account
        let link_as_account = Account::from_public_key(&block.link.as_public_key())
            .as_str()
            .to_string();

        let process_block = ProcessBlock {
            block_type: "state".to_string(),
            account: block.account.as_str().to_string(),
            previous: block.previous.to_hex(),
            representative: block.representative.as_str().to_string(),
            balance: block.balance.to_string(),
            link: block.link.to_hex(),
            link_as_account,
            signature: block
                .signature
                .map(|s| hex::encode_upper(s.as_bytes()))
                .unwrap_or_default(),
            work: block.work.map(|w| w.to_hex()).unwrap_or_default(),
        };

        ProcessRequest {
            action: "process".to_string(),
            json_block: Some("true".to_string()),
            subtype,
            block: process_block,
        }
    }
}

/// RPC action for work_generate.
#[derive(Debug, Serialize)]
pub struct WorkGenerateRequest {
    /// The RPC action name.
    pub action: String,
    /// The hash to generate work for.
    pub hash: String,
    /// Optional custom difficulty threshold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<String>,
    /// Use work peers for distributed generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_peers: Option<bool>,
    /// API key for RPC providers with authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl WorkGenerateRequest {
    /// Create a new work_generate request.
    pub fn new(hash: &BlockHash) -> Self {
        WorkGenerateRequest {
            action: "work_generate".to_string(),
            hash: hash.to_hex(),
            difficulty: None,
            use_peers: None,
            key: None,
        }
    }

    /// Set a custom difficulty threshold.
    pub fn with_difficulty(mut self, difficulty: &str) -> Self {
        self.difficulty = Some(difficulty.to_string());
        self
    }

    /// Enable distributed work generation using peers.
    pub fn use_peers(mut self) -> Self {
        self.use_peers = Some(true);
        self
    }

    /// Set an API key for authentication if required.
    pub fn with_key(mut self, key: &str) -> Self {
        self.key = Some(key.to_string());
        self
    }
}

/// RPC action for work_validate.
#[derive(Debug, Serialize)]
pub struct WorkValidateRequest {
    /// The RPC action name.
    pub action: String,
    /// The hash the work was generated for.
    pub hash: String,
    /// The work value to validate.
    pub work: String,
}

impl WorkValidateRequest {
    /// Create a new work_validate request.
    pub fn new(hash: &BlockHash, work: Work) -> Self {
        WorkValidateRequest {
            action: "work_validate".to_string(),
            hash: hash.to_hex(),
            work: work.to_hex(),
        }
    }
}

/// RPC action for work_cancel.
#[derive(Debug, Serialize)]
pub struct WorkCancelRequest {
    /// The RPC action name.
    pub action: String,
    /// The hash to cancel work generation for.
    pub hash: String,
}

impl WorkCancelRequest {
    /// Create a new work_cancel request.
    pub fn new(hash: &BlockHash) -> Self {
        WorkCancelRequest {
            action: "work_cancel".to_string(),
            hash: hash.to_hex(),
        }
    }
}

/// RPC action for version.
#[derive(Debug, Serialize)]
pub struct VersionRequest {
    /// The RPC action name.
    pub action: String,
}

impl VersionRequest {
    /// Create a new version request.
    pub fn new() -> Self {
        VersionRequest {
            action: "version".to_string(),
        }
    }
}

impl Default for VersionRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// RPC action for peers.
#[derive(Debug, Serialize)]
pub struct PeersRequest {
    /// The RPC action name.
    pub action: String,
}

impl PeersRequest {
    /// Create a new peers request.
    pub fn new() -> Self {
        PeersRequest {
            action: "peers".to_string(),
        }
    }
}

impl Default for PeersRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// RPC action for telemetry.
#[derive(Debug, Serialize)]
pub struct TelemetryRequest {
    /// The RPC action name.
    pub action: String,
}

impl TelemetryRequest {
    /// Create a new telemetry request.
    pub fn new() -> Self {
        TelemetryRequest {
            action: "telemetry".to_string(),
        }
    }
}

impl Default for TelemetryRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// RPC action for representatives.
#[derive(Debug, Serialize)]
pub struct RepresentativesRequest {
    /// The RPC action name.
    pub action: String,
    /// Maximum number of representatives to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<String>,
    /// Sort by voting weight descending.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorting: Option<bool>,
}

impl RepresentativesRequest {
    /// Create a new representatives request with sorting enabled.
    pub fn new() -> Self {
        RepresentativesRequest {
            action: "representatives".to_string(),
            count: None,
            sorting: Some(true),
        }
    }

    /// Limit the number of representatives returned.
    pub fn with_count(mut self, count: u64) -> Self {
        self.count = Some(count.to_string());
        self
    }
}

impl Default for RepresentativesRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// RPC action for representatives_online.
#[derive(Debug, Serialize)]
pub struct RepresentativesOnlineRequest {
    /// The RPC action name.
    pub action: String,
    /// Include voting weight in response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<bool>,
}

impl RepresentativesOnlineRequest {
    /// Create a new representatives_online request with weight enabled.
    pub fn new() -> Self {
        RepresentativesOnlineRequest {
            action: "representatives_online".to_string(),
            weight: Some(true),
        }
    }
}

impl Default for RepresentativesOnlineRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// RPC action for available_supply.
#[derive(Debug, Serialize)]
pub struct AvailableSupplyRequest {
    /// The RPC action name.
    pub action: String,
}

impl AvailableSupplyRequest {
    /// Create a new available_supply request.
    pub fn new() -> Self {
        AvailableSupplyRequest {
            action: "available_supply".to_string(),
        }
    }
}

impl Default for AvailableSupplyRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// RPC action for frontier_count.
#[derive(Debug, Serialize)]
pub struct FrontierCountRequest {
    /// The RPC action name.
    pub action: String,
}

impl FrontierCountRequest {
    /// Create a new frontier_count request.
    pub fn new() -> Self {
        FrontierCountRequest {
            action: "frontier_count".to_string(),
        }
    }
}

impl Default for FrontierCountRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// RPC action for confirmation_quorum.
#[derive(Debug, Serialize)]
pub struct ConfirmationQuorumRequest {
    /// The RPC action name.
    pub action: String,
}

impl ConfirmationQuorumRequest {
    /// Create a new confirmation_quorum request.
    pub fn new() -> Self {
        ConfirmationQuorumRequest {
            action: "confirmation_quorum".to_string(),
        }
    }
}

impl Default for ConfirmationQuorumRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// RPC action for block_confirm.
#[derive(Debug, Serialize)]
pub struct BlockConfirmRequest {
    /// The RPC action name.
    pub action: String,
    /// The block hash to request confirmation for.
    pub hash: String,
}

impl BlockConfirmRequest {
    /// Create a new block_confirm request.
    pub fn new(hash: &BlockHash) -> Self {
        BlockConfirmRequest {
            action: "block_confirm".to_string(),
            hash: hash.to_hex(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PublicKey, Raw, Signature, Subtype};

    fn test_account() -> Account {
        Account::from_public_key(
            &PublicKey::from_hex(
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            )
            .unwrap(),
        )
    }

    fn test_block_hash() -> BlockHash {
        BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
            .unwrap()
    }

    fn test_work() -> Work {
        Work::from_hex("FE00000000000000").unwrap()
    }

    #[test]
    fn test_account_balance_request() {
        let request = AccountBalanceRequest::new(&test_account());
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"account_balance\""));
        assert!(json.contains("nano_"));
    }

    #[test]
    fn test_account_info_request() {
        let request = AccountInfoRequest::new(&test_account());
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"account_info\""));
        assert!(json.contains("\"representative\":true"));
        assert!(json.contains("\"weight\":true"));
        assert!(json.contains("\"receivable\":true"));
    }

    #[test]
    fn test_account_history_request() {
        let request = AccountHistoryRequest::new(&test_account(), 100);
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"account_history\""));
        assert!(json.contains("\"count\":\"100\""));
    }

    #[test]
    fn test_account_history_request_with_head() {
        let request = AccountHistoryRequest::new(&test_account(), 50).with_head(&test_block_hash());
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(
            "\"head\":\"991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948\""
        ));
    }

    #[test]
    fn test_account_history_request_with_offset() {
        let request = AccountHistoryRequest::new(&test_account(), 50).with_offset(10);
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"offset\":10"));
    }

    #[test]
    fn test_account_history_request_reversed() {
        let request = AccountHistoryRequest::new(&test_account(), 50).reversed();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"reverse\":true"));
    }

    #[test]
    fn test_accounts_receivable_request() {
        let accounts = [test_account()];
        let request = AccountsReceivableRequest::new(&accounts, 10);
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"accounts_receivable\""));
        assert!(json.contains("\"count\":\"10\""));
        assert!(json.contains("\"source\":true"));
    }

    #[test]
    fn test_accounts_receivable_request_with_threshold() {
        let accounts = [test_account()];
        let request = AccountsReceivableRequest::new(&accounts, 10).with_threshold("1000000");
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"threshold\":\"1000000\""));
    }

    #[test]
    fn test_block_info_request() {
        let request = BlockInfoRequest::new(&test_block_hash());
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"block_info\""));
        assert!(json.contains("\"json_block\":true"));
    }

    #[test]
    fn test_block_count_request() {
        let request = BlockCountRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"block_count\""));
    }

    #[test]
    fn test_block_count_request_default() {
        let request = BlockCountRequest::default();
        assert_eq!(request.action, "block_count");
    }

    #[test]
    fn test_process_request() {
        use crate::types::Link;
        use core::str::FromStr;
        let block = StateBlock {
            block_type: "state".to_string(),
            account: test_account(),
            previous: test_block_hash(),
            representative: test_account(),
            balance: Raw::from_str("1000000000000000000000000000000").unwrap(),
            link: Link::from_bytes([0u8; 32]),
            signature: Some(Signature::from_bytes([0u8; 64])),
            work: Some(test_work()),
            subtype: Some(Subtype::Send),
        };
        let request = ProcessRequest::new(block);
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"process\""));
        assert!(json.contains("\"json_block\":\"true\""));
        assert!(json.contains("\"subtype\":\"send\""));
    }

    #[test]
    fn test_work_generate_request() {
        let request = WorkGenerateRequest::new(&test_block_hash());
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"work_generate\""));
        assert!(!json.contains("\"difficulty\""));
        assert!(!json.contains("\"use_peers\""));
        assert!(!json.contains("\"key\""));
    }

    #[test]
    fn test_work_generate_request_with_difficulty() {
        let request =
            WorkGenerateRequest::new(&test_block_hash()).with_difficulty("fffffff800000000");
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"difficulty\":\"fffffff800000000\""));
    }

    #[test]
    fn test_work_generate_request_use_peers() {
        let request = WorkGenerateRequest::new(&test_block_hash()).use_peers();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"use_peers\":true"));
    }

    #[test]
    fn test_work_generate_request_with_key() {
        let request = WorkGenerateRequest::new(&test_block_hash()).with_key("my_api_key");
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"key\":\"my_api_key\""));
    }

    #[test]
    fn test_work_validate_request() {
        let request = WorkValidateRequest::new(&test_block_hash(), test_work());
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"work_validate\""));
        assert!(json.contains("\"work\":\"fe00000000000000\""));
    }

    #[test]
    fn test_work_cancel_request() {
        let request = WorkCancelRequest::new(&test_block_hash());
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"work_cancel\""));
    }

    #[test]
    fn test_version_request() {
        let request = VersionRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"version\""));
    }

    #[test]
    fn test_version_request_default() {
        let request = VersionRequest::default();
        assert_eq!(request.action, "version");
    }

    #[test]
    fn test_peers_request() {
        let request = PeersRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"peers\""));
    }

    #[test]
    fn test_peers_request_default() {
        let request = PeersRequest::default();
        assert_eq!(request.action, "peers");
    }

    #[test]
    fn test_telemetry_request() {
        let request = TelemetryRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"telemetry\""));
    }

    #[test]
    fn test_telemetry_request_default() {
        let request = TelemetryRequest::default();
        assert_eq!(request.action, "telemetry");
    }

    #[test]
    fn test_representatives_request() {
        let request = RepresentativesRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"representatives\""));
        assert!(json.contains("\"sorting\":true"));
    }

    #[test]
    fn test_representatives_request_with_count() {
        let request = RepresentativesRequest::new().with_count(10);
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"count\":\"10\""));
    }

    #[test]
    fn test_representatives_request_default() {
        let request = RepresentativesRequest::default();
        assert_eq!(request.action, "representatives");
    }

    #[test]
    fn test_representatives_online_request() {
        let request = RepresentativesOnlineRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"representatives_online\""));
        assert!(json.contains("\"weight\":true"));
    }

    #[test]
    fn test_representatives_online_request_default() {
        let request = RepresentativesOnlineRequest::default();
        assert_eq!(request.action, "representatives_online");
    }

    #[test]
    fn test_available_supply_request() {
        let request = AvailableSupplyRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"available_supply\""));
    }

    #[test]
    fn test_available_supply_request_default() {
        let request = AvailableSupplyRequest::default();
        assert_eq!(request.action, "available_supply");
    }

    #[test]
    fn test_frontier_count_request() {
        let request = FrontierCountRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"frontier_count\""));
    }

    #[test]
    fn test_frontier_count_request_default() {
        let request = FrontierCountRequest::default();
        assert_eq!(request.action, "frontier_count");
    }

    #[test]
    fn test_confirmation_quorum_request() {
        let request = ConfirmationQuorumRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"confirmation_quorum\""));
    }

    #[test]
    fn test_confirmation_quorum_request_default() {
        let request = ConfirmationQuorumRequest::default();
        assert_eq!(request.action, "confirmation_quorum");
    }

    #[test]
    fn test_block_confirm_request() {
        let request = BlockConfirmRequest::new(&test_block_hash());
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"action\":\"block_confirm\""));
    }
}
