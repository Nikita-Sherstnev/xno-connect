//! Integration test for real Nano transfer with representative change.
//!
//! Run with: cargo test --features full --release real_transfer -- --ignored

#![cfg(not(target_arch = "wasm32"))]

use std::env;
use xno_connect::{rpc::RpcClient, types::Account, wallet::Wallet};

#[tokio::test]
#[ignore]
#[cfg(feature = "work-cpu")]
#[cfg(not(coverage))]
async fn test_send_and_change_representative() {
    dotenvy::dotenv().ok();

    let client = RpcClient::new(&env::var("NANO_RPC_URL").unwrap());
    let destination =
        Account::from_address_str_checked(&env::var("NANO_DESTINATION").unwrap()).unwrap();
    let new_rep =
        Account::from_address_str_checked(&env::var("NANO_REPRESENTATIVE").unwrap()).unwrap();

    let mut wallet = Wallet::from_hex_seed(&env::var("NANO_SEED").unwrap()).unwrap();
    let result = wallet
        .account(0)
        .send_and_change_local(&destination, 1.into(), &new_rep, &client)
        .await;

    println!("Result: {:?}", result);
    assert!(result.is_ok(), "Transfer failed: {:?}", result.err());
}
