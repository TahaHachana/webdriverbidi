use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::SessionError;
use crate::webdriver::capabilities::CapabilitiesRequest;

/// Represents the information returned when starting a WebDriver session.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: String,
    pub capabilities: CapabilitiesRequest,
    pub websocket_url: String,
}

/// Start a WebDriver session through HTTP.
pub async fn start_session(
    base_url: &str,
    capabilities: &CapabilitiesRequest,
) -> Result<SessionResponse, SessionError> {
    let url = format!("{}/session", base_url);
    let payload = capabilities.build();
    let client = create_http_client();

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send HTTP request: {}", e);
            SessionError::HttpRequestError(e)
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| {
            error!("Failed to parse JSON response: {}", e);
            SessionError::HttpRequestError(e)
        })?;

    // Extract sessionId and WebSocket URL
    let session_id = response["value"]["sessionId"].as_str().ok_or_else(|| {
        let msg = format!("JSON doesn't contain a sessionId field: {:?}", response);
        error!("{}", msg);
        SessionError::SessionResponseError(msg)
    })?;
    let websocket_url = response["value"]["capabilities"]["webSocketUrl"]
        .as_str()
        .ok_or_else(|| {
            let msg = format!("JSON doesn't contain a webSocketUrl field: {:?}", response);
            error!("{}", msg);
            SessionError::SessionResponseError(msg)
        })?;

    let session_response = SessionResponse {
        session_id: session_id.to_string(),
        capabilities: serde_json::from_value(response["value"]["capabilities"].clone()).map_err(
            |e| {
                let msg = format!("Failed to deserialize capabilities: {}", e);
                error!("{}", msg);
                SessionError::SessionResponseError(msg)
            },
        )?,
        websocket_url: websocket_url.to_string(),
    };

    debug!("Session started successfully: {:?}", session_response);
    Ok(session_response)
}

/// Close a WebDriver session through HTTP.
pub async fn close_session(base_url: &str, session_id: &str) -> Result<(), SessionError> {
    let url = format!("{}/session/{}", base_url, session_id);
    let client = create_http_client();

    client.delete(&url).send().await.map_err(|e| {
        error!("Failed to send HTTP request: {}", e);
        SessionError::HttpRequestError(e)
    })?;

    debug!("Session {} closed successfully", session_id);
    Ok(())
}

/// Create a new reqwest HTTP client.
fn create_http_client() -> Client {
    Client::new()
}
