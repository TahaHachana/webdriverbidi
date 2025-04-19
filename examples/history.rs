use anyhow::Result;
use tokio::time;

use webdriverbidi::model::browsing_context::{
    GetTreeParameters, NavigateParameters, ReadinessState, TraverseHistoryParameters,
};
use webdriverbidi::session::WebDriverBiDiSession;
use webdriverbidi::webdriver::capabilities::CapabilitiesRequest;

const HOST: &str = "localhost";
const PORT: u16 = 4444;

async fn sleep_for_secs(secs: u64) {
    time::sleep(time::Duration::from_secs(secs)).await
}

/// Initialize a new WebDriver BiDi session.
pub async fn init_session() -> Result<WebDriverBiDiSession> {
    let capabilities = CapabilitiesRequest::default();
    let mut session = WebDriverBiDiSession::new(HOST.into(), PORT, capabilities);
    session.start().await?;
    Ok(session)
}

/// Retrieve the browsing context at the specified index.
pub async fn get_context(session: &mut WebDriverBiDiSession, idx: usize) -> Result<String> {
    let get_tree_params = GetTreeParameters::new(None, None);
    let get_tree_rslt = session.browsing_context_get_tree(get_tree_params).await?;
    if let Some(context_entry) = get_tree_rslt.contexts.get(idx) {
        Ok(context_entry.context.clone())
    } else {
        anyhow::bail!(
            "No browsing context found at index {}. Available contexts: {}",
            idx,
            get_tree_rslt.contexts.len()
        );
    }
}

/// Navigate to the specified URL and wait for the document to completely load.
pub async fn navigate(session: &mut WebDriverBiDiSession, ctx: String, url: String) -> Result<()> {
    let navigate_params = NavigateParameters::new(ctx, url, Some(ReadinessState::Complete));
    session.browsing_context_navigate(navigate_params).await?;
    Ok(())
}

/// Navigate back or forward in the browsing history based on the provided delta value.
async fn traverse_history(
    session: &mut WebDriverBiDiSession,
    ctx: String,
    delta: i64,
) -> Result<()> {
    let traverse_history_params = TraverseHistoryParameters::new(ctx, delta);
    session
        .browsing_context_traverse_history(traverse_history_params)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut session = init_session().await?;
    let ctx = get_context(&mut session, 0).await?;

    // Load rust-lang.org
    let url = String::from("https://www.rust-lang.org/");
    navigate(&mut session, ctx.clone(), url).await?;
    sleep_for_secs(1).await;

    // Load crates.io
    let url = String::from("https://crates.io");
    navigate(&mut session, ctx.clone(), url).await?;
    sleep_for_secs(1).await;

    // Go back to rust-lang.org
    traverse_history(&mut session, ctx.clone(), -1).await?;
    sleep_for_secs(1).await;

    // Go forward to crates.io
    traverse_history(&mut session, ctx, 1).await?;
    sleep_for_secs(1).await;

    // Close the session
    session.close().await?;
    Ok(())
}
