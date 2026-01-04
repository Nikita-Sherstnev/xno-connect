//! WebSocket message types.

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::types::{Account, BlockHash, Raw, Signature, Work};

/// Outgoing WebSocket message (subscription request).
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeMessage {
    /// Action type.
    pub action: String,
    /// Topic to subscribe to.
    pub topic: String,
    /// Acknowledgement flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack: Option<bool>,
    /// Options for the subscription.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<SubscriptionOptions>,
}

/// Subscription options.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SubscriptionOptions {
    /// Filter by accounts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<String>>,
    /// Include block contents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_block: Option<bool>,
    /// Include election info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_election_info: Option<bool>,
}

/// Incoming WebSocket message wrapper.
#[derive(Debug, Clone, Deserialize)]
pub struct IncomingMessage {
    /// Topic of the message.
    pub topic: String,
    /// Timestamp.
    #[serde(default)]
    pub time: Option<String>,
    /// Message content.
    pub message: serde_json::Value,
}

/// Acknowledgement message.
#[derive(Debug, Clone, Deserialize)]
pub struct AckMessage {
    /// Acknowledgement flag.
    pub ack: String,
    /// Timestamp.
    #[serde(default)]
    pub time: Option<String>,
    /// ID if provided.
    #[serde(default)]
    pub id: Option<String>,
}

/// Confirmation message content.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfirmationMessage {
    /// Account involved.
    pub account: Account,
    /// Amount transferred.
    pub amount: Raw,
    /// Block hash.
    pub hash: BlockHash,
    /// Confirmation type.
    pub confirmation_type: String,
    /// Block contents (if requested).
    #[serde(default)]
    pub block: Option<ConfirmationBlock>,
    /// Election info (if requested).
    #[serde(default)]
    pub election_info: Option<ElectionInfo>,
}

/// Block within a confirmation message.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfirmationBlock {
    /// Block type.
    #[serde(rename = "type")]
    pub block_type: String,
    /// Account.
    pub account: Account,
    /// Previous block.
    pub previous: BlockHash,
    /// Representative.
    pub representative: Account,
    /// Balance.
    pub balance: Raw,
    /// Link field.
    pub link: String,
    /// Link as account.
    #[serde(default)]
    pub link_as_account: Option<Account>,
    /// Signature.
    pub signature: Signature,
    /// Work.
    pub work: Work,
    /// Subtype.
    #[serde(default)]
    pub subtype: Option<String>,
}

/// Election info within a confirmation message.
#[derive(Debug, Clone, Deserialize)]
pub struct ElectionInfo {
    /// Election duration.
    pub duration: String,
    /// Confirmation request count.
    pub request_count: String,
    /// Blocks in election.
    pub blocks: String,
    /// Voters.
    pub voters: String,
    /// Tally.
    #[serde(default)]
    pub tally: Option<Raw>,
}

/// Vote message content.
#[derive(Debug, Clone, Deserialize)]
pub struct VoteMessage {
    /// Account that voted.
    pub account: Account,
    /// Signature of the vote.
    pub signature: String,
    /// Sequence number.
    pub sequence: String,
    /// Timestamp.
    pub timestamp: String,
    /// Blocks voted on.
    pub blocks: Vec<String>,
}

/// Stopped election message.
#[derive(Debug, Clone, Deserialize)]
pub struct StoppedElectionMessage {
    /// Block hash.
    pub hash: BlockHash,
}

/// Active difficulty message.
#[derive(Debug, Clone, Deserialize)]
pub struct ActiveDifficultyMessage {
    /// Network minimum difficulty.
    pub network_minimum: String,
    /// Network current difficulty.
    pub network_current: String,
    /// Network receive minimum.
    #[serde(default)]
    pub network_receive_minimum: Option<String>,
    /// Network receive current.
    #[serde(default)]
    pub network_receive_current: Option<String>,
    /// Multiplier.
    pub multiplier: String,
}

/// Telemetry message.
#[derive(Debug, Clone, Deserialize)]
pub struct TelemetryMessage {
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
    /// Address.
    #[serde(default)]
    pub address: Option<String>,
    /// Port.
    #[serde(default)]
    pub port: Option<String>,
}

/// Work message (for work_generate responses via WS).
#[derive(Debug, Clone, Deserialize)]
pub struct WorkMessage {
    /// Success flag.
    pub success: String,
    /// Hash.
    pub hash: BlockHash,
    /// Generated work.
    #[serde(default)]
    pub work: Option<Work>,
    /// Difficulty achieved.
    #[serde(default)]
    pub difficulty: Option<String>,
    /// Multiplier.
    #[serde(default)]
    pub multiplier: Option<String>,
}

/// Parse an incoming message into a typed enum.
#[derive(Debug, Clone)]
pub enum ParsedMessage {
    /// Confirmation message.
    Confirmation(ConfirmationMessage),
    /// Vote message.
    Vote(VoteMessage),
    /// Stopped election.
    StoppedElection(StoppedElectionMessage),
    /// Active difficulty update.
    ActiveDifficulty(ActiveDifficultyMessage),
    /// Telemetry update.
    Telemetry(TelemetryMessage),
    /// Work generation result.
    Work(WorkMessage),
    /// Unknown message type.
    Unknown(IncomingMessage),
}

impl IncomingMessage {
    /// Parse the message into a typed variant.
    pub fn parse(self) -> ParsedMessage {
        match self.topic.as_str() {
            "confirmation" => {
                if let Ok(msg) = serde_json::from_value(self.message.clone()) {
                    ParsedMessage::Confirmation(msg)
                } else {
                    ParsedMessage::Unknown(self)
                }
            }
            "vote" => {
                if let Ok(msg) = serde_json::from_value(self.message.clone()) {
                    ParsedMessage::Vote(msg)
                } else {
                    ParsedMessage::Unknown(self)
                }
            }
            "stopped_election" => {
                if let Ok(msg) = serde_json::from_value(self.message.clone()) {
                    ParsedMessage::StoppedElection(msg)
                } else {
                    ParsedMessage::Unknown(self)
                }
            }
            "active_difficulty" => {
                if let Ok(msg) = serde_json::from_value(self.message.clone()) {
                    ParsedMessage::ActiveDifficulty(msg)
                } else {
                    ParsedMessage::Unknown(self)
                }
            }
            "telemetry" => {
                if let Ok(msg) = serde_json::from_value(self.message.clone()) {
                    ParsedMessage::Telemetry(msg)
                } else {
                    ParsedMessage::Unknown(self)
                }
            }
            "work" => {
                if let Ok(msg) = serde_json::from_value(self.message.clone()) {
                    ParsedMessage::Work(msg)
                } else {
                    ParsedMessage::Unknown(self)
                }
            }
            _ => ParsedMessage::Unknown(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_message_serialization() {
        let msg = SubscribeMessage {
            action: "subscribe".to_string(),
            topic: "confirmation".to_string(),
            ack: Some(true),
            options: Some(SubscriptionOptions {
                accounts: Some(vec![
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3".to_string(),
                ]),
                include_block: Some(true),
                include_election_info: None,
            }),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("subscribe"));
        assert!(json.contains("confirmation"));
        assert!(json.contains("accounts"));
    }

    #[test]
    fn test_confirmation_message_deserialization() {
        let json = r#"{
            "topic": "confirmation",
            "time": "1234567890",
            "message": {
                "account": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
                "amount": "1000000000000000000000000000000",
                "hash": "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
                "confirmation_type": "active_quorum"
            }
        }"#;

        let msg: IncomingMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.topic, "confirmation");

        if let ParsedMessage::Confirmation(conf) = msg.parse() {
            assert_eq!(conf.confirmation_type, "active_quorum");
        } else {
            panic!("Expected Confirmation message");
        }
    }

    #[test]
    fn test_ack_message_deserialization() {
        let json = r#"{"ack": "subscribe", "time": "1234567890", "id": "1"}"#;
        let msg: AckMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.ack, "subscribe");
    }
}
