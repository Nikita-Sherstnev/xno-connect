//! WebSocket subscription management.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::types::Account;
use crate::websocket::messages::{SubscribeMessage, SubscriptionOptions};

/// WebSocket topic for subscriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    /// Block confirmations.
    Confirmation,
    /// Votes.
    Vote,
    /// Stopped elections.
    StoppedElection,
    /// Active difficulty changes.
    ActiveDifficulty,
    /// Work generation results.
    Work,
    /// Telemetry updates.
    Telemetry,
    /// New unconfirmed blocks.
    NewUnconfirmedBlock,
    /// Bootstrap updates.
    Bootstrap,
}

impl Topic {
    /// Get the topic string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Topic::Confirmation => "confirmation",
            Topic::Vote => "vote",
            Topic::StoppedElection => "stopped_election",
            Topic::ActiveDifficulty => "active_difficulty",
            Topic::Work => "work",
            Topic::Telemetry => "telemetry",
            Topic::NewUnconfirmedBlock => "new_unconfirmed_block",
            Topic::Bootstrap => "bootstrap",
        }
    }
}

impl core::fmt::Display for Topic {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Builder for creating subscription requests.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionBuilder {
    topic: Option<Topic>,
    ack: bool,
    accounts: Vec<String>,
    include_block: bool,
    include_election_info: bool,
}

impl SubscriptionBuilder {
    /// Create a new subscription builder.
    pub fn new() -> Self {
        SubscriptionBuilder::default()
    }

    /// Set the topic to subscribe to.
    pub fn topic(mut self, topic: Topic) -> Self {
        self.topic = Some(topic);
        self
    }

    /// Subscribe to confirmations.
    pub fn confirmations(mut self) -> Self {
        self.topic = Some(Topic::Confirmation);
        self
    }

    /// Subscribe to votes.
    pub fn votes(mut self) -> Self {
        self.topic = Some(Topic::Vote);
        self
    }

    /// Subscribe to telemetry.
    pub fn telemetry(mut self) -> Self {
        self.topic = Some(Topic::Telemetry);
        self
    }

    /// Request acknowledgement.
    pub fn with_ack(mut self) -> Self {
        self.ack = true;
        self
    }

    /// Filter by account.
    pub fn account(mut self, account: &Account) -> Self {
        self.accounts.push(account.as_str().to_string());
        self
    }

    /// Filter by multiple accounts.
    pub fn accounts(mut self, accounts: &[Account]) -> Self {
        for account in accounts {
            self.accounts.push(account.as_str().to_string());
        }
        self
    }

    /// Include block contents in confirmations.
    pub fn include_block(mut self) -> Self {
        self.include_block = true;
        self
    }

    /// Include election info in confirmations.
    pub fn include_election_info(mut self) -> Self {
        self.include_election_info = true;
        self
    }

    /// Build the subscribe message.
    pub fn build_subscribe(self) -> Option<SubscribeMessage> {
        let topic = self.topic?;

        let options =
            if self.accounts.is_empty() && !self.include_block && !self.include_election_info {
                None
            } else {
                Some(SubscriptionOptions {
                    accounts: if self.accounts.is_empty() {
                        None
                    } else {
                        Some(self.accounts)
                    },
                    include_block: if self.include_block { Some(true) } else { None },
                    include_election_info: if self.include_election_info {
                        Some(true)
                    } else {
                        None
                    },
                })
            };

        Some(SubscribeMessage {
            action: "subscribe".to_string(),
            topic: topic.as_str().to_string(),
            ack: if self.ack { Some(true) } else { None },
            options,
        })
    }

    /// Build the unsubscribe message.
    pub fn build_unsubscribe(self) -> Option<SubscribeMessage> {
        let topic = self.topic?;

        Some(SubscribeMessage {
            action: "unsubscribe".to_string(),
            topic: topic.as_str().to_string(),
            ack: if self.ack { Some(true) } else { None },
            options: None,
        })
    }
}

/// Shorthand for creating a confirmation subscription.
pub fn subscribe_confirmations() -> SubscriptionBuilder {
    SubscriptionBuilder::new().confirmations()
}

/// Shorthand for creating a confirmation subscription for specific accounts.
pub fn subscribe_account_confirmations(accounts: &[Account]) -> SubscriptionBuilder {
    SubscriptionBuilder::new()
        .confirmations()
        .accounts(accounts)
        .include_block()
}

/// Shorthand for creating a vote subscription.
pub fn subscribe_votes() -> SubscriptionBuilder {
    SubscriptionBuilder::new().votes()
}

/// Shorthand for creating a telemetry subscription.
pub fn subscribe_telemetry() -> SubscriptionBuilder {
    SubscriptionBuilder::new().telemetry()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PublicKey;

    #[test]
    fn test_topic_as_str() {
        assert_eq!(Topic::Confirmation.as_str(), "confirmation");
        assert_eq!(Topic::Vote.as_str(), "vote");
        assert_eq!(Topic::Telemetry.as_str(), "telemetry");
    }

    #[test]
    fn test_subscription_builder_basic() {
        let msg = SubscriptionBuilder::new()
            .confirmations()
            .with_ack()
            .build_subscribe()
            .unwrap();

        assert_eq!(msg.action, "subscribe");
        assert_eq!(msg.topic, "confirmation");
        assert_eq!(msg.ack, Some(true));
    }

    #[test]
    fn test_subscription_builder_with_accounts() {
        let account = Account::from_public_key(
            &PublicKey::from_hex(
                "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            )
            .unwrap(),
        );

        let msg = SubscriptionBuilder::new()
            .confirmations()
            .account(&account)
            .include_block()
            .build_subscribe()
            .unwrap();

        assert!(msg.options.is_some());
        let opts = msg.options.unwrap();
        assert!(opts.accounts.is_some());
        assert_eq!(opts.include_block, Some(true));
    }

    #[test]
    fn test_unsubscribe() {
        let msg = SubscriptionBuilder::new()
            .confirmations()
            .build_unsubscribe()
            .unwrap();

        assert_eq!(msg.action, "unsubscribe");
        assert_eq!(msg.topic, "confirmation");
    }

    #[test]
    fn test_shorthand_functions() {
        let msg = subscribe_confirmations().build_subscribe().unwrap();
        assert_eq!(msg.topic, "confirmation");

        let msg = subscribe_votes().build_subscribe().unwrap();
        assert_eq!(msg.topic, "vote");

        let msg = subscribe_telemetry().build_subscribe().unwrap();
        assert_eq!(msg.topic, "telemetry");
    }
}
