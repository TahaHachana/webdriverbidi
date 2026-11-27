//! Live tests for the `browsingContext` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use anyhow::{Result, ensure};
use webdriverbidi::model::browsing_context::{
    CaptureScreenshotParameters, CloseParameters, ContextCreated, CreateParameters, CreateType,
    GetTreeParameters, Load, StartScreencastParameters, StopScreencastParameters,
};

use support::{inline, navigate, subscribe, supported, top_context, with_session};

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/navigate
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/load
#[tokio::test]
async fn navigate_fires_load_and_updates_tree() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let url = inline("<title>navigate</title><p>hello</p>");
        let mut loads = subscribe(&mut session, "browsingContext.load").await?;

        navigate(&mut session, &context, &url).await?;

        let load: Load = loads
            .next_matching(|event: &Load| event.params.base.url == url)
            .await?;
        ensure!(
            load.params.base.context == context,
            "load event for another context"
        );
        let tree = session
            .browsing_context_get_tree(GetTreeParameters::new(None, Some(context.clone())))
            .await?;
        ensure!(
            tree.contexts[0].url == url,
            "getTree reports {}",
            tree.contexts[0].url
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/create
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/close
#[tokio::test]
async fn create_and_close_tab() -> Result<()> {
    with_session(|mut session| async move {
        let mut created = subscribe(&mut session, "browsingContext.contextCreated").await?;

        let tab = session
            .browsing_context_create(CreateParameters::new(CreateType::Tab, None, None, None))
            .await?
            .context;

        created
            .next_matching(|event: &ContextCreated| event.params.info.context == tab)
            .await?;
        let tree = session
            .browsing_context_get_tree(GetTreeParameters::new(Some(0), None))
            .await?;
        ensure!(
            tree.contexts.iter().any(|info| info.context == tab),
            "new tab missing from getTree"
        );

        session
            .browsing_context_close(CloseParameters::new(tab.clone(), None))
            .await?;
        let tree = session
            .browsing_context_get_tree(GetTreeParameters::new(Some(0), None))
            .await?;
        ensure!(
            tree.contexts.iter().all(|info| info.context != tab),
            "closed tab still in getTree"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/capture_screenshot
#[tokio::test]
async fn capture_screenshot_returns_png() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(
            &mut session,
            &context,
            &inline("<p style='font-size:40px'>shot</p>"),
        )
        .await?;
        let screenshot = session
            .browsing_context_capture_screenshot(CaptureScreenshotParameters::new(
                context, None, None, None, None,
            ))
            .await?;
        // Base64 of the PNG signature.
        ensure!(
            screenshot.data.starts_with("iVBORw0KGgo"),
            "not a PNG: {}…",
            &screenshot.data[..screenshot.data.len().min(16)]
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/start_screencast
#[tokio::test]
async fn screencast_start_and_stop() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let started = session
            .browsing_context_start_screencast(StartScreencastParameters::new(
                context, None, None, None, None,
            ))
            .await;
        let Some(started) = supported("browsingContext.startScreencast", started)? else {
            return Ok(());
        };
        ensure!(!started.path.is_empty(), "screencast path is empty");
        session
            .browsing_context_stop_screencast(StopScreencastParameters::new(started.screencast))
            .await?;
        Ok(())
    })
    .await
}
