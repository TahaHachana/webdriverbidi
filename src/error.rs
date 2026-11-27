use thiserror::Error;

use crate::model::error::{ErrorCode, ErrorResponse};
use tokio_tungstenite::tungstenite;

/// Errors that can occur when sending a WebDriver command.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CommandError {
    /// Error during JSON serialization/deserialization.
    #[error("Serialization error: {0}.")]
    SerdeError(#[from] serde_json::Error),

    /// Missing command ID field in the command.
    #[error("Missing command ID field.")]
    MissingCommandId,

    /// Error when sending data over a WebSocket. Boxed because
    /// `tungstenite::Error` is large.
    #[error("WebSocket send error: {0}.")]
    WebSocketSendError(Box<tungstenite::Error>),

    /// Missing result field in the response.
    #[error("Missing result field.")]
    MissingResult,

    /// The remote end answered with an error response (raw JSON). Use
    /// [`CommandError::error_response`] or [`CommandError::error_code`] for the
    /// typed form.
    #[error("{}", describe_error_response(.0))]
    Error(serde_json::Value),

    /// Timeout when waiting for a response.
    #[error("Timeout waiting for receiver response")]
    TimeoutError,

    /// The connection to the remote end closed before the command was sent or
    /// before its response arrived.
    #[error("The connection to the remote end is closed.")]
    ConnectionClosed,

    /// Other command errors.
    #[error("Command error: {0}.")]
    Other(String),
}

impl CommandError {
    /// The remote end's error response, if this error is one.
    pub fn error_response(&self) -> Option<ErrorResponse> {
        match self {
            CommandError::Error(json) => serde_json::from_value(json.clone()).ok(),
            _ => None,
        }
    }

    /// The spec error code (e.g. [`ErrorCode::NoSuchFrame`]), if the remote end
    /// answered with an error response.
    pub fn error_code(&self) -> Option<ErrorCode> {
        match self {
            CommandError::Error(json) => json
                .get("error")
                .and_then(serde_json::Value::as_str)
                .map(ErrorCode::from),
            _ => None,
        }
    }
}

/// `"<code>: <message>"` for an error response, or the raw JSON otherwise.
fn describe_error_response(json: &serde_json::Value) -> String {
    match (
        json.get("error").and_then(serde_json::Value::as_str),
        json.get("message").and_then(serde_json::Value::as_str),
    ) {
        (Some(code), Some(message)) => format!("Command returned error `{code}`: {message}"),
        _ => format!("Command returned error: {json}"),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn error_responses_are_typed_and_readable() {
        let error = CommandError::Error(json!({
            "type": "error", "id": 3, "error": "no such frame",
            "message": "Context not found", "stacktrace": "at f()"
        }));
        assert_eq!(error.error_code(), Some(ErrorCode::NoSuchFrame));
        let response = error.error_response().unwrap();
        assert_eq!(response.id, Some(3));
        assert_eq!(response.message, "Context not found");
        assert_eq!(
            error.to_string(),
            "Command returned error `no such frame`: Context not found"
        );
        assert_eq!(CommandError::ConnectionClosed.error_code(), None);
    }
}
