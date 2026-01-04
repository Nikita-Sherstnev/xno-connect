//! WebSocket integration test.
//!
//! Run with: NANO_WS_URL=ws://localhost:7078 cargo test --features full --release websocket -- --ignored --nocapture

#![cfg(not(target_arch = "wasm32"))]

use std::env;
use std::time::Duration;
use xno_connect::{
    types::Account,
    websocket::{ParsedMessage, SubscriptionBuilder, WebSocketClient},
};

#[tokio::test]
#[ignore]
async fn test_websocket_connect_and_subscribe() {
    dotenvy::dotenv().ok();

    let ws_url = match env::var("NANO_WS_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("NANO_WS_URL not set, skipping WebSocket test");
            return;
        }
    };

    // Connect to WebSocket
    let mut client = WebSocketClient::connect(&ws_url)
        .await
        .expect("Failed to connect to WebSocket");

    println!("Connected to {}", client.url());

    // Subscribe to confirmations
    client
        .subscribe(
            SubscriptionBuilder::new()
                .confirmations()
                .with_ack()
                .include_block(),
        )
        .await
        .expect("Failed to subscribe");

    println!("Subscribed to confirmations - OK");

    // Close connection
    client.close().await.expect("Failed to close connection");

    println!("Test passed! Connection and subscription work.");
}

#[tokio::test]
#[ignore]
// Funds should be send to nano destination address to receive confirmation
async fn test_websocket_receive_with_timeout() {
    dotenvy::dotenv().ok();

    let ws_url = match env::var("NANO_WS_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("NANO_WS_URL not set, skipping WebSocket test");
            return;
        }
    };

    let mut client = WebSocketClient::connect(&ws_url)
        .await
        .expect("Failed to connect");

    let account =
        Account::from_address_str_checked(&env::var("NANO_DESTINATION").unwrap()).unwrap();
    client
        .subscribe(
            SubscriptionBuilder::new()
                .confirmations()
                .with_ack()
                .account(&account),
        )
        .await
        .expect("Failed to subscribe");

    println!("Waiting for messages (10 second timeout)...");

    let mut count = 0;
    let timeout_duration = Duration::from_secs(10);

    // Try to receive up to 3 messages with timeout
    while count < 3 {
        match tokio::time::timeout(timeout_duration, client.receive()).await {
            Ok(Ok(Some(ParsedMessage::Confirmation(conf)))) => {
                println!("Confirmation: {} ({} raw)", conf.hash.to_hex(), conf.amount);
                count += 1;
            }
            Ok(Ok(Some(other))) => {
                println!("Other message: {:?}", other);
            }
            Ok(Ok(None)) => {
                println!("Connection closed");
                break;
            }
            Ok(Err(e)) => {
                println!("Error: {:?}", e);
                break;
            }
            Err(_) => {
                println!("Timeout - no more messages");
                break;
            }
        }
    }

    client.close().await.ok();

    println!("Received {} confirmation(s)", count);
    println!("Test passed!");
}
