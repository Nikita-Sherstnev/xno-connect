//! RPC client for communicating with Nano nodes.

use alloc::string::{String, ToString};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{Error, Result, RpcError};
use crate::rpc::requests::*;
use crate::rpc::responses::*;
use crate::types::{Account, BlockHash, StateBlock, Work};

/// Asynchronous RPC client for Nano node communication.
///
/// Uses `reqwest` for non-blocking HTTP requests. Works on both native and WASM.
///
/// # Example
///
/// ```no_run
/// use xno_connect::rpc::RpcClient;
///
/// # async fn example() -> xno_connect::error::Result<()> {
/// let client = RpcClient::new("http://localhost:7076");
/// let account = "nano_1abc...".parse()?;
/// let balance = client.account_balance(&account).await?;
/// println!("Balance: {}", balance.balance);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RpcClient {
    url: String,
    client: reqwest::Client,
}

impl RpcClient {
    /// Create a new RPC client.
    pub fn new(url: impl Into<String>) -> Self {
        RpcClient {
            url: url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Get the node URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Send a raw RPC request.
    async fn request<Req: Serialize, Resp: DeserializeOwned>(&self, request: &Req) -> Result<Resp> {
        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| {
                Error::Rpc(RpcError::ConnectionFailed(alloc::format!(
                    "{}: {}", &self.url, e
                )))
            })?;

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::Rpc(RpcError::InvalidResponse(e.to_string())))?;

        if let Some(error) = check_error(&json) {
            return Err(Error::Rpc(RpcError::NodeError(error)));
        }

        serde_json::from_value(json)
            .map_err(|e| Error::Rpc(RpcError::InvalidResponse(e.to_string())))
    }

    /// Get account balance.
    pub async fn account_balance(&self, account: &Account) -> Result<AccountBalanceResponse> {
        self.request(&AccountBalanceRequest::new(account)).await
    }

    /// Get account info.
    pub async fn account_info(&self, account: &Account) -> Result<AccountInfoResponse> {
        self.request(&AccountInfoRequest::new(account)).await
    }

    /// Get account history.
    pub async fn account_history(
        &self,
        account: &Account,
        count: u64,
    ) -> Result<AccountHistoryResponse> {
        self.request(&AccountHistoryRequest::new(account, count))
            .await
    }

    /// Get account history with pagination.
    pub async fn account_history_from(
        &self,
        account: &Account,
        count: u64,
        head: &BlockHash,
    ) -> Result<AccountHistoryResponse> {
        self.request(&AccountHistoryRequest::new(account, count).with_head(head))
            .await
    }

    /// Get receivable blocks for accounts.
    pub async fn accounts_receivable(
        &self,
        accounts: &[Account],
        count: u64,
    ) -> Result<AccountsReceivableResponse> {
        self.request(&AccountsReceivableRequest::new(accounts, count))
            .await
    }

    /// Get block info.
    pub async fn block_info(&self, hash: &BlockHash) -> Result<BlockInfoResponse> {
        self.request(&BlockInfoRequest::new(hash)).await
    }

    /// Get block count.
    pub async fn block_count(&self) -> Result<BlockCountResponse> {
        self.request(&BlockCountRequest::new()).await
    }

    /// Request block confirmation.
    pub async fn block_confirm(&self, hash: &BlockHash) -> Result<()> {
        let _: serde_json::Value = self.request(&BlockConfirmRequest::new(hash)).await?;
        Ok(())
    }

    /// Process (submit) a block.
    pub async fn process(&self, block: StateBlock) -> Result<ProcessResponse> {
        self.request(&ProcessRequest::new(block)).await
    }

    /// Generate work via the node.
    pub async fn work_generate(&self, hash: &BlockHash) -> Result<WorkGenerateResponse> {
        self.request(&WorkGenerateRequest::new(hash)).await
    }

    /// Generate work with custom difficulty.
    pub async fn work_generate_with_difficulty(
        &self,
        hash: &BlockHash,
        difficulty: &str,
    ) -> Result<WorkGenerateResponse> {
        self.request(&WorkGenerateRequest::new(hash).with_difficulty(difficulty))
            .await
    }

    /// Generate work with an API key (for providers with authentication).
    pub async fn work_generate_with_key(
        &self,
        hash: &BlockHash,
        key: &str,
    ) -> Result<WorkGenerateResponse> {
        self.request(&WorkGenerateRequest::new(hash).with_key(key))
            .await
    }

    /// Validate work.
    pub async fn work_validate(&self, hash: &BlockHash, work: Work) -> Result<bool> {
        #[derive(serde::Deserialize)]
        struct ValidateResponse {
            valid_all: Option<String>,
            valid: Option<String>,
        }
        let response: ValidateResponse =
            self.request(&WorkValidateRequest::new(hash, work)).await?;
        let valid = response.valid_all.or(response.valid).unwrap_or_default();
        Ok(valid == "1")
    }

    /// Cancel pending work generation.
    pub async fn work_cancel(&self, hash: &BlockHash) -> Result<()> {
        let _: serde_json::Value = self.request(&WorkCancelRequest::new(hash)).await?;
        Ok(())
    }

    /// Get node version info.
    pub async fn version(&self) -> Result<VersionResponse> {
        self.request(&VersionRequest::new()).await
    }

    /// Get connected peers.
    pub async fn peers(&self) -> Result<PeersResponse> {
        self.request(&PeersRequest::new()).await
    }

    /// Get network telemetry.
    pub async fn telemetry(&self) -> Result<TelemetryResponse> {
        self.request(&TelemetryRequest::new()).await
    }

    /// Get representatives and their voting weight.
    pub async fn representatives(&self) -> Result<RepresentativesResponse> {
        self.request(&RepresentativesRequest::new()).await
    }

    /// Get top representatives by weight.
    pub async fn representatives_top(&self, count: u64) -> Result<RepresentativesResponse> {
        self.request(&RepresentativesRequest::new().with_count(count))
            .await
    }

    /// Get online representatives.
    pub async fn representatives_online(&self) -> Result<RepresentativesOnlineResponse> {
        self.request(&RepresentativesOnlineRequest::new()).await
    }

    /// Get available supply.
    pub async fn available_supply(&self) -> Result<AvailableSupplyResponse> {
        self.request(&AvailableSupplyRequest::new()).await
    }

    /// Get frontier (account) count.
    pub async fn frontier_count(&self) -> Result<FrontierCountResponse> {
        self.request(&FrontierCountRequest::new()).await
    }

    /// Get confirmation quorum info.
    pub async fn confirmation_quorum(&self) -> Result<ConfirmationQuorumResponse> {
        self.request(&ConfirmationQuorumRequest::new()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local_rpc_url() -> String {
        dotenvy::dotenv().ok();
        std::env::var("LOCAL_NANO_RPC_URL").unwrap_or_else(|_| "http://localhost:7076".to_string())
    }

    fn remote_rpc_url() -> String {
        dotenvy::dotenv().ok();
        std::env::var("NANO_RPC_URL").unwrap_or_else(|_| "https://rpc.nano.to".to_string())
    }

    fn rpc_key() -> Option<String> {
        dotenvy::dotenv().ok();
        std::env::var("NANO_RPC_KEY").ok()
    }

    fn local_client() -> RpcClient {
        RpcClient::new(local_rpc_url())
    }

    fn remote_client() -> RpcClient {
        RpcClient::new(remote_rpc_url())
    }

    fn genesis_account() -> Account {
        Account::from_address_str_checked(
            "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
        )
        .unwrap()
    }

    fn genesis_block() -> BlockHash {
        BlockHash::from_hex("991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948")
            .unwrap()
    }

    fn first_block() -> BlockHash {
        BlockHash::from_hex("A170D51B94E00371ACE76E35AC81DC9405D5D04D4CEBC399AEACE07AE05DD293")
            .unwrap()
    }

    fn state_block() -> BlockHash {
        BlockHash::from_hex("1155DA8DECD1B706782072190833F687D49C003D8BDE3CAF3C9952002C9008FF")
            .unwrap()
    }

    #[test]
    fn test_client_creation() {
        let client = RpcClient::new("https://example.com");
        assert_eq!(client.url(), "https://example.com");
    }

    #[test]
    fn test_request_serialization() {
        let account = Account::from_public_key(
            &crate::types::PublicKey::from_hex(
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            )
            .unwrap(),
        );

        let request = AccountBalanceRequest::new(&account);
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("account_balance"));
        assert!(json.contains("nano_"));
    }

    #[tokio::test]
    async fn test_account_balance() {
        let client = local_client();
        let account = genesis_account();
        let balance = client.account_balance(&account).await.unwrap();
        assert!(!balance.balance.is_zero());
    }

    #[tokio::test]
    async fn test_account_info() {
        let client = local_client();
        let account = genesis_account();
        let info = client.account_info(&account).await.unwrap();
        assert!(!info.balance.is_zero());
    }

    #[tokio::test]
    async fn test_account_history() {
        let client = local_client();
        let account = genesis_account();
        let history = client.account_history(&account, 10).await.unwrap();
        assert!(!history.history.is_empty());
    }

    #[tokio::test]
    async fn test_account_history_from() {
        let client = local_client();
        let account = genesis_account();
        let block = genesis_block();
        let history = client
            .account_history_from(&account, 10, &block)
            .await
            .unwrap();
        assert_eq!(history.account, account);
    }

    #[tokio::test]
    async fn test_accounts_receivable() {
        let client = local_client();
        let accounts = [genesis_account()];
        let receivable = client.accounts_receivable(&accounts, 10).await.unwrap();
        assert!(receivable.blocks.contains_key(accounts[0].as_str()));
    }

    #[tokio::test]
    async fn test_genesis_block_info() {
        let client = local_client();
        let block = genesis_block();
        let info = client.block_info(&block).await.unwrap();
        assert_eq!(info.block_account, genesis_account());
    }

    #[tokio::test]
    async fn test_block_info() {
        let client = remote_client();
        let block = state_block();
        let block_info = client.block_info(&block).await.unwrap();
        let expected_balance = "33000000000000000000000000000";
        assert_eq!(block_info.contents.balance.unwrap(), expected_balance);
    }

    #[tokio::test]
    async fn test_block_count() {
        let client = local_client();
        let count = client.block_count().await.unwrap();
        assert!(!count.count.is_empty());
    }

    #[tokio::test]
    async fn test_block_confirm() {
        let client = local_client();
        let block = genesis_block();
        let _ = client.block_confirm(&block).await;
    }

    #[tokio::test]
    async fn test_work_validate() {
        let client = local_client();
        let block_info = client.block_info(&first_block()).await.unwrap();
        let work = block_info.contents.work;
        let previous = block_info.contents.previous.unwrap();
        let result = client.work_validate(&previous, work).await.unwrap();
        // False against the real node, because now difficulty is higher
        assert_eq!(result, false);
    }

    #[tokio::test]
    async fn test_work_generate_with_key() {
        let client = remote_client();
        let hash = state_block();
        let key = rpc_key().unwrap();
        let result = client.work_generate_with_key(&hash, &key).await.unwrap();
        assert!(!result.work.is_zero());
    }

    #[tokio::test]
    async fn test_version() {
        let client = local_client();
        let version = client.version().await.unwrap();
        assert!(!version.node_vendor.is_empty());
    }

    #[tokio::test]
    async fn test_peers() {
        let client = local_client();
        let result = client.peers().await.unwrap();
        let _ = result.peers;
    }

    #[tokio::test]
    async fn test_telemetry() {
        let client = local_client();
        let telemetry = client.telemetry().await.unwrap();
        assert!(!telemetry.block_count.is_empty());
    }

    #[tokio::test]
    async fn test_representatives() {
        let client = local_client();
        let result = client.representatives().await.unwrap();
        let _ = result.representatives;
    }

    #[tokio::test]
    async fn test_representatives_top() {
        let client = local_client();
        let result = client.representatives_top(5).await.unwrap();
        let _ = result.representatives;
    }

    #[tokio::test]
    async fn test_representatives_online() {
        let client = local_client();
        let reps = client.representatives_online().await.unwrap();
        assert!(!reps.representatives.is_null());
    }

    #[tokio::test]
    async fn test_available_supply() {
        let client = local_client();
        let result = client.available_supply().await.unwrap();
        assert!(!result.available.is_zero());
    }

    #[tokio::test]
    async fn test_frontier_count() {
        let client = local_client();
        let result = client.frontier_count().await.unwrap();
        assert!(result.count.parse::<u64>().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_confirmation_quorum() {
        let client = local_client();
        let quorum = client.confirmation_quorum().await.unwrap();
        assert!(!quorum.quorum_delta.is_zero());
    }

    #[tokio::test]
    async fn test_check_error_with_error() {
        let json: serde_json::Value = serde_json::json!({"error": "Account not found"});
        let error = check_error(&json);
        assert_eq!(error, Some("Account not found".to_string()));
    }

    #[tokio::test]
    async fn test_check_error_without_error() {
        let json: serde_json::Value = serde_json::json!({"balance": "100"});
        let error = check_error(&json);
        assert!(error.is_none());
    }

    #[tokio::test]
    async fn test_work_generate() {
        let client = local_client();
        let hash = genesis_block();
        let result = client.work_generate(&hash).await.unwrap();
        assert!(!result.work.is_zero());
    }

    #[tokio::test]
    async fn test_work_generate_with_difficulty() {
        let client = local_client();
        let hash = genesis_block();
        let result = client
            .work_generate_with_difficulty(&hash, "fffffff800000000")
            .await
            .unwrap();
        assert!(!result.work.is_zero());
    }

    #[tokio::test]
    async fn test_work_cancel() {
        let client = local_client();
        let hash = first_block();
        let _ = client.work_generate(&hash);
        let _ = client.work_cancel(&hash).await.unwrap();
    }

    #[tokio::test]
    async fn test_process_block() {
        use crate::types::{Link, Raw, Signature, Subtype};
        use core::str::FromStr;

        let client = remote_client();
        // Test invalid block
        let block = StateBlock {
            block_type: "state".to_string(),
            account: genesis_account(),
            previous: genesis_block(),
            representative: genesis_account(),
            balance: Raw::from_str("0").unwrap(),
            link: Link::from_bytes([0u8; 32]),
            signature: Some(Signature::from_bytes([0u8; 64])),
            work: Some(Work::from_hex("0000000000000000").unwrap()),
            subtype: Some(Subtype::Send),
        };
        let _ = client.process(block).await;
    }

    #[tokio::test]
    async fn test_connection_error() {
        let client = RpcClient::new("http://localhost:1");
        let account = genesis_account();
        let result = client.account_balance(&account).await;
        assert!(result.is_err());
        if let Err(Error::Rpc(RpcError::ConnectionFailed(msg))) = result {
            assert!(msg.contains("localhost:1"));
        }
    }

    #[tokio::test]
    async fn test_node_error() {
        let client = local_client();
        let invalid_account = Account::from_public_key(
            &crate::types::PublicKey::from_hex(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
        );
        let _ = client.account_balance(&invalid_account).await;
    }
}
