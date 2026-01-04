//! Error types for the XNO-connect library.

use alloc::string::String;
use core::fmt;

/// Result type alias for XNO-connect operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Error types that can occur in XNO-connect operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Invalid seed format or length.
    InvalidSeed,
    /// Invalid private key format or length.
    InvalidPrivateKey,
    /// Invalid public key format or length.
    InvalidPublicKey,
    /// Invalid account address format or checksum.
    InvalidAccount(AccountError),
    /// Invalid block hash format or length.
    InvalidBlockHash,
    /// Invalid block structure or missing fields.
    InvalidBlock(BlockError),
    /// Invalid signature format or verification failed.
    InvalidSignature,
    /// Invalid work value or insufficient difficulty.
    InvalidWork,
    /// Invalid amount value or overflow.
    InvalidAmount(AmountError),
    /// Hex decoding error.
    HexDecode(HexError),
    /// RPC communication error.
    #[cfg(any(feature = "rpc", feature = "wasm-rpc"))]
    Rpc(RpcError),
    /// WebSocket communication error.
    #[cfg(any(feature = "websocket", feature = "wasm-websocket"))]
    WebSocket(WebSocketError),
    /// Work generation error.
    WorkGeneration(WorkError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidSeed => write!(f, "invalid seed: must be 32 bytes"),
            Error::InvalidPrivateKey => write!(f, "invalid private key: must be 32 bytes"),
            Error::InvalidPublicKey => write!(f, "invalid public key: must be 32 bytes"),
            Error::InvalidAccount(e) => write!(f, "invalid account: {}", e),
            Error::InvalidBlockHash => write!(f, "invalid block hash: must be 32 bytes"),
            Error::InvalidBlock(e) => write!(f, "invalid block: {}", e),
            Error::InvalidSignature => write!(f, "invalid signature"),
            Error::InvalidWork => write!(f, "invalid work: insufficient difficulty"),
            Error::InvalidAmount(e) => write!(f, "invalid amount: {}", e),
            Error::HexDecode(e) => write!(f, "hex decode error: {}", e),
            #[cfg(any(feature = "rpc", feature = "wasm-rpc"))]
            Error::Rpc(e) => write!(f, "RPC error: {}", e),
            #[cfg(any(feature = "websocket", feature = "wasm-websocket"))]
            Error::WebSocket(e) => write!(f, "WebSocket error: {}", e),
            Error::WorkGeneration(e) => write!(f, "work generation error: {}", e),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidAccount(e) => Some(e),
            Error::InvalidBlock(e) => Some(e),
            Error::InvalidAmount(e) => Some(e),
            Error::HexDecode(e) => Some(e),
            #[cfg(any(feature = "rpc", feature = "wasm-rpc"))]
            Error::Rpc(e) => Some(e),
            #[cfg(any(feature = "websocket", feature = "wasm-websocket"))]
            Error::WebSocket(e) => Some(e),
            Error::WorkGeneration(e) => Some(e),
            _ => None,
        }
    }
}

/// Account-specific error details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountError {
    /// Invalid prefix (must be "nano_" or "xno_").
    InvalidPrefix,
    /// Invalid length for account string.
    InvalidLength,
    /// Invalid base32 encoding.
    InvalidEncoding,
    /// Checksum mismatch.
    ChecksumMismatch,
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountError::InvalidPrefix => write!(f, "invalid prefix (expected 'nano_' or 'xno_')"),
            AccountError::InvalidLength => write!(f, "invalid length"),
            AccountError::InvalidEncoding => write!(f, "invalid base32 encoding"),
            AccountError::ChecksumMismatch => write!(f, "checksum mismatch"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AccountError {}

/// Block-specific error details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    /// Missing required field.
    MissingField(&'static str),
    /// Invalid subtype for the operation.
    InvalidSubtype,
    /// Invalid link field.
    InvalidLink,
    /// Previous block hash mismatch.
    PreviousMismatch,
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockError::MissingField(field) => write!(f, "missing required field: {}", field),
            BlockError::InvalidSubtype => write!(f, "invalid block subtype"),
            BlockError::InvalidLink => write!(f, "invalid link field"),
            BlockError::PreviousMismatch => write!(f, "previous block hash mismatch"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for BlockError {}

/// Amount-specific error details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmountError {
    /// Value overflow.
    Overflow,
    /// Invalid string format.
    InvalidFormat,
    /// Negative value not allowed.
    Negative,
}

impl fmt::Display for AmountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AmountError::Overflow => write!(f, "amount overflow"),
            AmountError::InvalidFormat => write!(f, "invalid format"),
            AmountError::Negative => write!(f, "negative values not allowed"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AmountError {}

/// Hex decoding error details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexError {
    /// Invalid character in hex string.
    InvalidCharacter,
    /// Invalid length for hex string.
    InvalidLength,
}

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HexError::InvalidCharacter => write!(f, "invalid character"),
            HexError::InvalidLength => write!(f, "invalid length"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for HexError {}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        match e {
            hex::FromHexError::InvalidHexCharacter { .. } => {
                Error::HexDecode(HexError::InvalidCharacter)
            }
            hex::FromHexError::OddLength | hex::FromHexError::InvalidStringLength => {
                Error::HexDecode(HexError::InvalidLength)
            }
        }
    }
}

/// RPC-specific error details.
#[cfg(any(feature = "rpc", feature = "wasm-rpc"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RpcError {
    /// Connection failed.
    ConnectionFailed(String),
    /// Request timeout.
    Timeout,
    /// Invalid response format.
    InvalidResponse(String),
    /// Node returned an error.
    NodeError(String),
    /// HTTP status error.
    HttpStatus(u16),
}

#[cfg(any(feature = "rpc", feature = "wasm-rpc"))]
impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcError::ConnectionFailed(msg) => write!(f, "connection failed: {}", msg),
            RpcError::Timeout => write!(f, "request timeout"),
            RpcError::InvalidResponse(msg) => write!(f, "invalid response: {}", msg),
            RpcError::NodeError(msg) => write!(f, "node error: {}", msg),
            RpcError::HttpStatus(code) => write!(f, "HTTP status: {}", code),
        }
    }
}

#[cfg(all(any(feature = "rpc", feature = "wasm-rpc"), feature = "std"))]
impl std::error::Error for RpcError {}

/// WebSocket-specific error details.
#[cfg(any(feature = "websocket", feature = "wasm-websocket"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketError {
    /// Connection failed.
    ConnectionFailed(String),
    /// Connection closed unexpectedly.
    ConnectionClosed,
    /// Invalid message format.
    InvalidMessage(String),
    /// Subscription failed.
    SubscriptionFailed(String),
}

#[cfg(any(feature = "websocket", feature = "wasm-websocket"))]
impl fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebSocketError::ConnectionFailed(msg) => write!(f, "connection failed: {}", msg),
            WebSocketError::ConnectionClosed => write!(f, "connection closed"),
            WebSocketError::InvalidMessage(msg) => write!(f, "invalid message: {}", msg),
            WebSocketError::SubscriptionFailed(msg) => write!(f, "subscription failed: {}", msg),
        }
    }
}

#[cfg(all(
    any(feature = "websocket", feature = "wasm-websocket"),
    feature = "std"
))]
impl std::error::Error for WebSocketError {}

/// Work generation error details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkError {
    /// Work generation cancelled.
    Cancelled,
    /// Maximum iterations reached without finding valid work.
    MaxIterations,
    /// External work server error.
    ServerError(String),
}

impl fmt::Display for WorkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkError::Cancelled => write!(f, "work generation cancelled"),
            WorkError::MaxIterations => write!(f, "max iterations reached"),
            WorkError::ServerError(msg) => write!(f, "server error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WorkError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        assert_eq!(
            Error::InvalidSeed.to_string(),
            "invalid seed: must be 32 bytes"
        );
        assert_eq!(
            Error::InvalidAccount(AccountError::ChecksumMismatch).to_string(),
            "invalid account: checksum mismatch"
        );
        assert_eq!(
            Error::InvalidBlock(BlockError::MissingField("balance")).to_string(),
            "invalid block: missing required field: balance"
        );
    }

    #[test]
    fn test_account_error_display() {
        assert_eq!(
            AccountError::InvalidPrefix.to_string(),
            "invalid prefix (expected 'nano_' or 'xno_')"
        );
        assert_eq!(AccountError::InvalidLength.to_string(), "invalid length");
        assert_eq!(
            AccountError::InvalidEncoding.to_string(),
            "invalid base32 encoding"
        );
        assert_eq!(
            AccountError::ChecksumMismatch.to_string(),
            "checksum mismatch"
        );
    }

    #[test]
    fn test_amount_error_display() {
        assert_eq!(AmountError::Overflow.to_string(), "amount overflow");
        assert_eq!(AmountError::InvalidFormat.to_string(), "invalid format");
        assert_eq!(
            AmountError::Negative.to_string(),
            "negative values not allowed"
        );
    }

    #[test]
    fn test_work_error_display() {
        assert_eq!(
            WorkError::Cancelled.to_string(),
            "work generation cancelled"
        );
        assert_eq!(
            WorkError::MaxIterations.to_string(),
            "max iterations reached"
        );
        assert_eq!(
            WorkError::ServerError("timeout".to_string()).to_string(),
            "server error: timeout"
        );
    }
}
