use anyhow::Result;
use tokio::time;

use webdriverbidi::model::browsing_context::{
    GetTreeParameters, NavigateParameters, ReadinessState,
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

#[tokio::main]
async fn main() -> Result<()> {
    let mut session = init_session().await?;
    let ctx = get_context(&mut session, 0).await?;

    let url = String::from("https://www.rust-lang.org/");
    navigate(&mut session, ctx, url).await?;

    sleep_for_secs(1).await;
    session.close().await?;
    Ok(())
}
