use crate::webdriver::capabilities::Capabilities;
use std::error::Error;

/// Represents the session response
pub struct SessionResponse {
    pub session_id: String,
    pub capabilities: serde_json::Value,
    pub websocket_url: String,
}

/// Starts a WebDriver session
pub async fn start_session(
    base_url: &String,
    capabilities: &Capabilities,
) -> Result<SessionResponse, Box<dyn std::error::Error>> {
    let url = format!("{}/session", base_url);

    let payload = capabilities.build();

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    println!("{:?}", response);

    // Extract sessionId and WebSocket URL
    let session_id = response["value"]["sessionId"]
        .as_str()
        .ok_or("Missing sessionId")?;
    let websocket_url = response["value"]["capabilities"]["webSocketUrl"]
        .as_str()
        .ok_or("Missing webSocketUrl")?;

    // Check if the browser window is open
    let window_check_url = format!("{}/session/{}/window", base_url, session_id);
    let window_response = client
        .get(&window_check_url)
        .send()
        .await?
        .error_for_status()?
        .json::<serde_json::Value>()
        .await?;

    if window_response["value"].is_null() {
        return Err("Browser window is not open".into());
    }

    Ok(SessionResponse {
        session_id: session_id.to_string(),
        capabilities: response["value"]["capabilities"].clone(),
        websocket_url: websocket_url.to_string(),
    })
}

/// Closes an existing WebDriver session
pub async fn close_session(base_url: &str, session_id: &str) -> Result<(), Box<dyn Error>> {
    // Create the HTTP endpoint for closing the session
    let url = format!("{}/session/{}", base_url, session_id);

    // Make the DELETE request to close the session
    let client = reqwest::Client::new();
    client.delete(&url).send().await?.error_for_status()?;
    Ok(())
}