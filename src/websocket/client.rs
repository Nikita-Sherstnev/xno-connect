//! WebSocket client for Nano node communication.

use alloc::string::{String, ToString};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite_wasm::{connect, Message, WebSocketStream};

use crate::error::{Error, Result, WebSocketError};
use crate::websocket::messages::{IncomingMessage, ParsedMessage, SubscribeMessage};
use crate::websocket::subscription::SubscriptionBuilder;

/// Asynchronous WebSocket client for real-time Nano node updates.
///
/// Uses `tokio-tungstenite-wasm` for unified native + WASM support.
///
/// # Example
///
/// ```no_run
/// use xno_connect::websocket::{WebSocketClient, SubscriptionBuilder, ParsedMessage};
///
/// # async fn example() -> xno_connect::error::Result<()> {
/// let mut client = WebSocketClient::connect("ws://localhost:7078").await?;
///
/// // Subscribe to confirmations
/// client.subscribe(SubscriptionBuilder::new().confirmations().with_ack()).await?;
///
/// // Receive messages
/// while let Some(msg) = client.receive().await? {
///     match msg {
///         ParsedMessage::Confirmation(conf) => {
///             println!("Confirmed: {} -> {}", conf.account, conf.amount);
///         }
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct WebSocketClient {
    stream: WebSocketStream,
    url: String,
}

impl WebSocketClient {
    /// Connect to a Nano node WebSocket endpoint.
    ///
    /// # Arguments
    /// * `url` - WebSocket URL (e.g., "ws://localhost:7078")
    pub async fn connect(url: impl Into<String>) -> Result<Self> {
        let url = url.into();
        let stream = connect(&url)
            .await
            .map_err(|e| Error::WebSocket(WebSocketError::ConnectionFailed(e.to_string())))?;

        Ok(WebSocketClient { stream, url })
    }

    /// Get the WebSocket URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Send a subscription message.
    pub async fn subscribe(&mut self, builder: SubscriptionBuilder) -> Result<()> {
        let msg = builder.build_subscribe().ok_or_else(|| {
            Error::WebSocket(WebSocketError::SubscriptionFailed(
                "no topic specified".to_string(),
            ))
        })?;
        self.send_message(&msg).await
    }

    /// Send an unsubscribe message.
    pub async fn unsubscribe(&mut self, builder: SubscriptionBuilder) -> Result<()> {
        let msg = builder.build_unsubscribe().ok_or_else(|| {
            Error::WebSocket(WebSocketError::SubscriptionFailed(
                "no topic specified".to_string(),
            ))
        })?;
        self.send_message(&msg).await
    }

    /// Send a raw message.
    async fn send_message(&mut self, msg: &SubscribeMessage) -> Result<()> {
        let json = serde_json::to_string(msg)
            .map_err(|e| Error::WebSocket(WebSocketError::InvalidMessage(e.to_string())))?;

        self.stream
            .send(Message::Text(json.into()))
            .await
            .map_err(|e| Error::WebSocket(WebSocketError::ConnectionFailed(e.to_string())))?;

        Ok(())
    }

    /// Receive the next message.
    ///
    /// Returns `Ok(Some(message))` on success, `Ok(None)` if the connection is closed.
    pub async fn receive(&mut self) -> Result<Option<ParsedMessage>> {
        loop {
            match self.stream.next().await {
                Some(Ok(msg)) => match msg {
                    Message::Text(text) => {
                        if let Ok(incoming) = serde_json::from_str::<IncomingMessage>(&text) {
                            return Ok(Some(incoming.parse()));
                        }
                        // Could be an ack message, skip
                        continue;
                    }
                    Message::Binary(_) => continue,
                    Message::Close(_) => {
                        return Ok(None);
                    }
                },
                Some(Err(e)) => {
                    return Err(Error::WebSocket(WebSocketError::ConnectionFailed(
                        e.to_string(),
                    )));
                }
                None => {
                    return Ok(None);
                }
            }
        }
    }

    /// Close the WebSocket connection.
    pub async fn close(mut self) -> Result<()> {
        self.stream
            .close()
            .await
            .map_err(|e| Error::WebSocket(WebSocketError::ConnectionFailed(e.to_string())))?;
        Ok(())
    }

    // /// Check if the connection is still open.
    // pub fn is_connected(&self) -> bool {
    //     self.stream.can_read() && self.stream.can_write()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_builder() {
        let builder = SubscriptionBuilder::new().confirmations().with_ack();

        let msg = builder.build_subscribe().unwrap();
        assert_eq!(msg.action, "subscribe");
        assert_eq!(msg.topic, "confirmation");
    }
}
