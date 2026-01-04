//! Integration test for creating and submitting a change block.
//!
//! This test:
//! 1. Creates an account from the all-zero seed (index 0)
//! 2. Fetches account info from a public RPC node
//! 3. Creates a change block to update the representative
//! 4. Computes proof of work locally
//! 5. Submits the block to the network
//! 6. Asserts the block was accepted
//!
//! Run with: cargo test --release --features full -- --ignored --nocapture

#![cfg(not(target_arch = "wasm32"))]

use xno_connect::blocks::create_change_block;
use xno_connect::keys::Seed;
use xno_connect::prelude::*;
#[cfg(feature = "work-cpu")]
use xno_connect::work::CpuWorkGenerator;

const RPC_URL: &str = "https://rpc.nano-gpt.com";
const ZERO_SEED: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const NEW_REPRESENTATIVE: &str =
    "nano_1natrium1o3z5519ifou7xii8crpxpk8y65qmkih8e8bpsjri651oza8imdd";

#[tokio::test]
#[ignore] // Run with: cargo test --release --features full -- --ignored --nocapture
#[cfg(feature = "work-cpu")]
#[cfg(not(coverage))]
async fn test_change_block_integration() {
    // Step 1: Create keypair from all-zero seed
    let seed = Seed::from_hex(ZERO_SEED).expect("Failed to parse zero seed");
    let keypair = seed.derive(0);
    let account = keypair.account();

    // Step 2: Create RPC client and fetch account info
    let client = RpcClient::new(RPC_URL);
    let account_info = client
        .account_info(&account)
        .await
        .expect("Failed to get account info");

    let frontier = account_info.frontier;
    let balance = account_info.balance;
    let current_rep = account_info
        .representative
        .map(|r| r.to_string())
        .unwrap_or_default();

    // Step 3: Determine the new representative (toggle to ensure a change)
    let new_rep = Account::from_address_str_checked(NEW_REPRESENTATIVE)
        .expect("Failed to parse new representative address");
    let final_rep = if current_rep == NEW_REPRESENTATIVE {
        account.clone()
    } else {
        new_rep
    };

    // Step 4: Compute proof of work on frontier hash
    let generator = CpuWorkGenerator::new();
    let work = generator
        .generate_for_subtype(&frontier, Subtype::Change)
        .expect("Failed to generate work");

    // Step 5: Create the signed change block with work
    let block = create_change_block(&keypair, frontier, final_rep, balance, Some(work));

    // Verify signature before sending
    assert!(
        xno_connect::blocks::BlockSigner::verify(&block),
        "Block signature verification failed"
    );

    // Step 6: Submit block to the network
    let response = client
        .process(block)
        .await
        .expect("Failed to submit block to network");

    // Assert the block was accepted
    assert!(
        !response.hash.is_zero(),
        "Expected non-zero block hash in response"
    );
}
