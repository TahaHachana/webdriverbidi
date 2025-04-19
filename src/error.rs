use thiserror::Error;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite;

/// Errors that can occur when sending a WebDriver command.
#[derive(Error, Debug)]
pub enum CommandError {
    /// Error during JSON serialization/deserialization.
    #[error("Serialization error: {0}.")]
    SerdeError(#[from] serde_json::Error),

    /// Missing command ID field in the command.
    #[error("Missing command ID field.")]
    MissingCommandId,

    /// Error when sending data over a WebSocket.
    #[error("WebSocket send error: {0}.")]
    WebSocketSendError(#[from] tungstenite::Error),

    /// Missing result field in the response.
    #[error("Missing result field.")]
    MissingResult,

    // Command returned an error.
    #[error("Command returned error: {0}")]
    Error(serde_json::Value),

    /// Timeout when waiting for a receiver response
    #[error("Timeout waiting for receiver response")]
    TimeoutError,

    /// Error when receiving a value from a one-shot channel.
    #[error("Oneshot receiver error: {0}.")]
    OneshotReceiverError(#[from] oneshot::error::RecvError),

    /// Other command errors.
    #[error("Command error: {0}.")]
    Other(String),
}

/// Errors that can occur when starting a WebDriver session.
#[derive(Error, Debug)]
pub enum SessionError {
    /// Error during an HTTP request.
    #[error("HTTP request error: {0}.")]
    HttpRequestError(#[from] reqwest::Error),

    /// Error in the session response.
    #[error("Session response error: {0}.")]
    SessionResponseError(String),

    /// Other command errors.
    #[error("Session error: {0}.")]
    Other(String),
}
