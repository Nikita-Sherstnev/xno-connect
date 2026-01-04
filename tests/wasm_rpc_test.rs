//! WASM integration test for real RPC transfer.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use xno_connect::blocks::{BlockBuilder, BlockSigner};
use xno_connect::keys::Seed;
use xno_connect::rpc::RpcClient;
use xno_connect::types::{Account, Raw, Subtype};

wasm_bindgen_test_configure!(run_in_browser);

// Read from environment at compile time
const RPC_URL: &str = match option_env!("NANO_RPC_URL") {
    Some(url) => url,
    None => "https://rpc.nano.to",
};

#[wasm_bindgen_test]
async fn test_wasm_version_rpc() {
    let client = RpcClient::new(RPC_URL);
    let version = client.version().await;

    assert!(version.is_ok(), "Version RPC failed: {:?}", version.err());
    let v = version.unwrap();
    assert!(!v.node_vendor.is_empty());
}

#[wasm_bindgen_test]
async fn test_wasm_block_count() {
    let client = RpcClient::new(RPC_URL);
    let count = client.block_count().await;

    assert!(count.is_ok(), "Block count RPC failed: {:?}", count.err());
    let c = count.unwrap();
    assert!(!c.count.is_empty());
}

#[wasm_bindgen_test]
async fn test_wasm_send_with_rep_change() {
    // Get config from compile-time env vars
    let seed_hex = match option_env!("NANO_SEED") {
        Some(s) => s,
        None => {
            // Skip test if not configured
            return;
        }
    };
    let destination_str = match option_env!("NANO_DESTINATION") {
        Some(s) => s,
        None => return,
    };
    let rep_str = match option_env!("NANO_REPRESENTATIVE") {
        Some(s) => s,
        None => return,
    };

    // Setup
    let seed = Seed::from_hex(seed_hex).expect("Invalid seed");
    let keypair = seed.derive(0);
    let account = keypair.account();
    let destination =
        Account::from_address_str_checked(destination_str).expect("Invalid destination");
    let new_rep = Account::from_address_str_checked(rep_str).expect("Invalid representative");

    let client = RpcClient::new(RPC_URL);

    // Get account info
    let info = client
        .account_info(&account)
        .await
        .expect("Failed to get account info");

    let current_balance = info.balance;
    let frontier = info.frontier;

    // Calculate new balance (subtract 1 raw)
    let new_balance = current_balance
        .checked_sub(Raw::new(1))
        .expect("Insufficient balance");

    // Get work for the frontier
    let work_response = client
        .work_generate(&frontier)
        .await
        .expect("Failed to generate work");

    // Build the send block with rep change
    let block = BlockBuilder::new()
        .account(account.clone())
        .previous(frontier)
        .representative(new_rep)
        .balance(new_balance)
        .link_as_account(&destination)
        .subtype(Subtype::Send)
        .work(work_response.work)
        .sign(&keypair)
        .build()
        .expect("Failed to build block");

    // Verify signature before sending
    assert!(BlockSigner::verify(&block), "Block signature invalid");

    // Submit the block
    let result = client.process(block).await;

    assert!(result.is_ok(), "Process RPC failed: {:?}", result.err());

    let hash = result.unwrap().hash;
    assert!(!hash.is_zero(), "Block hash should not be zero");
}
