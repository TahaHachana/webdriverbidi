use tokio;
use tokio::time;

// --------------------------------------------------

use webdriverbidi::remote::browsing_context::{
    GetTreeParameters, NavigateParameters, ReadinessState,
};
use webdriverbidi::session::WebDriverBiDiSession;
use webdriverbidi::webdriver::capabilities::{Capabilities, CapabilityRequest};

// --------------------------------------------------

async fn sleep(secs: u64) {
    time::sleep(time::Duration::from_secs(secs)).await
}

#[tokio::main]
async fn main() {
    // Define the capabilities for the WebDriver session
    let always_match = CapabilityRequest::new();
    let capabilities = Capabilities::new(always_match);

    // Initialize a new WebDriver BiDi session and start it
    let host = String::from("localhost");
    let port = 4444;
    let mut bidi_session = WebDriverBiDiSession::new(host, port, capabilities);
    let _ = bidi_session.start().await.expect("Failed to start session");

    // Get the browsing context tree
    let get_tree_params = GetTreeParameters::new(None, None);
    let get_tree_rslt = bidi_session
        .browsing_context_get_tree(get_tree_params)
        .await
        .expect("Failed to send command");

    // Navigate to rust-lang.org
    let navigate_params = NavigateParameters::new(
        get_tree_rslt.contexts[0].context.clone(),
        "https://www.rust-lang.org/".to_string(),
        Some(ReadinessState::Complete),
    );
    let _ = bidi_session
        .browsing_context_navigate(navigate_params)
        .await
        .expect("Failed to send command");

    sleep(2).await;

    // Close the session
    bidi_session.close().await.expect("Failed to close session");
}
