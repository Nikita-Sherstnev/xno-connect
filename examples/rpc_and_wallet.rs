use std::env;

use xno_connect::prelude::{Account, BlockHash, RpcClient, Wallet};

// Run
// cargo run --release --example rpc_and_wallet --features work-cpu
// in release mode, otherwise work generation will be very slow.
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // RpcClient gives access to all RPC methods
    let client = RpcClient::new("https://rpc.nano.to");

    let block_info = client
        .block_info(
            &BlockHash::from_hex(
                "993C64997382809B7B7EDB5303F32711F99B862D22A93DFD9556AF837476BD47",
            )
            .unwrap(),
        )
        .await
        .unwrap();

    println!("{:?}", block_info);

    let destination =
        Account::from_address_str_checked(&env::var("NANO_DESTINATION").unwrap()).unwrap();

    // Wallet provides access to multiple accounts derived from a single seed
    let mut wallet = Wallet::from_hex_seed(&env::var("NANO_SEED").unwrap()).unwrap();
    // _local means that work will be computed locally
    let result = wallet
        .account(0)
        .send_local(&destination, 1.into(), &client) // Send single raw
        .await;

    println!("Result: {:?}", result);
}
