//! Live tests for the `browsingContext` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use std::time::Duration;

use anyhow::{Result, anyhow, bail, ensure};
use webdriverbidi::model::browser::{DownloadBehavior, SetDownloadBehaviorParameters};
use webdriverbidi::model::browsing_context::*;
use webdriverbidi::session::WebDriverBiDiSession;

use support::{
    eval_string, evaluate, inline, is_firefox, navigate, server_url, subscribe,
    subscribe_if_supported, supported, temp_dir, top_context, with_session,
};

async fn get_tree(
    session: &mut WebDriverBiDiSession,
    max_depth: Option<u64>,
    root: Option<&str>,
) -> Result<InfoList> {
    Ok(session
        .browsing_context_get_tree(GetTreeParameters::new(max_depth, root.map(str::to_string)))
        .await?
        .contexts)
}

async fn new_tab(session: &mut WebDriverBiDiSession) -> Result<BrowsingContext> {
    Ok(session
        .browsing_context_create(CreateParameters::new(CreateType::Tab, None, None, None))
        .await?
        .context)
}

/// Poll `expression` until it evaluates to `expected`, for state that settles asynchronously.
async fn wait_for(
    session: &mut WebDriverBiDiSession,
    context: &str,
    expression: &str,
    expected: &str,
) -> Result<()> {
    let mut last = String::new();
    for _ in 0..50 {
        last = eval_string(session, context, expression).await?;
        if last == expected {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    bail!("`{expression}` is {last:?}, expected {expected:?}")
}

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
        let tree = get_tree(&mut session, None, Some(&context)).await?;
        ensure!(tree[0].url == url, "getTree reports {}", tree[0].url);
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/dom_content_loaded
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/navigation_started
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/navigation_committed
#[tokio::test]
async fn navigation_lifecycle_events() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let url = inline("<p>lifecycle</p>");
        let mut started = subscribe(&mut session, "browsingContext.navigationStarted").await?;
        let mut committed = subscribe(&mut session, "browsingContext.navigationCommitted").await?;
        let mut dom_loaded = subscribe(&mut session, "browsingContext.domContentLoaded").await?;

        let navigation = session
            .browsing_context_navigate(NavigateParameters::new(
                context.clone(),
                url.clone(),
                Some(ReadinessState::Complete),
            ))
            .await?
            .navigation;

        let started: NavigationStarted = started
            .next_matching(|e: &NavigationStarted| e.params.base.url == url)
            .await?;
        ensure!(
            started.params.base.navigation == navigation,
            "navigationStarted has navigation {:?}, navigate returned {navigation:?}",
            started.params.base.navigation
        );
        committed
            .next_matching(|e: &NavigationCommitted| e.params.base.url == url)
            .await?;
        dom_loaded
            .next_matching(|e: &DomContentLoaded| e.params.base.url == url)
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/fragment_navigated
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/history_updated
#[tokio::test]
async fn fragment_and_history_events() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let url = inline("<p id='section'>fragments</p>");
        navigate(&mut session, &context, &url).await?;
        let mut fragments = subscribe(&mut session, "browsingContext.fragmentNavigated").await?;
        let mut history = subscribe(&mut session, "browsingContext.historyUpdated").await?;

        let with_fragment = format!("{url}#section");
        navigate(&mut session, &context, &with_fragment).await?;
        fragments
            .next_matching(|e: &FragmentNavigated| e.params.base.url == with_fragment)
            .await?;

        evaluate(
            &mut session,
            &context,
            "history.pushState({}, '', '?pushed')",
        )
        .await?;
        let updated: HistoryUpdated = history
            .next_matching(|e: &HistoryUpdated| e.params.url.ends_with("?pushed"))
            .await?;
        ensure!(
            updated.params.context == context,
            "historyUpdated for another context"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/navigation_failed
#[tokio::test]
async fn navigation_failed_event() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let mut failed = subscribe(&mut session, "browsingContext.navigationFailed").await?;
        // Nothing listens on port 1.
        let url = "http://127.0.0.1:1/";
        let result = session
            .browsing_context_navigate(NavigateParameters::new(
                context.clone(),
                url.into(),
                Some(ReadinessState::Complete),
            ))
            .await;
        ensure!(result.is_err(), "navigating to a refused port succeeded");
        let event = failed
            .next_matching(|e: &NavigationFailed| e.params.base.url == url)
            .await;
        // Firefox 161 shows its error page without a navigationFailed event.
        match event {
            Err(e) if is_firefox() => {
                eprintln!(
                    "NOTE: known Firefox gap, no navigationFailed for a refused connection: {e}"
                )
            }
            other => {
                other?;
            }
        }
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/navigation_aborted
// Chrome 158 reports a navigation interrupted by another one as `navigationFailed`
// ("navigation canceled by concurrent navigation"), not `navigationAborted`, so
// either event is accepted.
#[tokio::test]
async fn interrupted_navigation_is_reported() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let mut started = subscribe(&mut session, "browsingContext.navigationStarted").await?;
        // Firefox 161 doesn't know the `navigationAborted` event.
        let mut aborted =
            subscribe_if_supported(&mut session, "browsingContext.navigationAborted").await?;
        let mut failed = subscribe(&mut session, "browsingContext.navigationFailed").await?;

        // Chrome answers `navigate` only once the navigation commits, so start the
        // slow one on another task and interrupt it while it's still in flight.
        let mut background = session.clone();
        let slow_url = server_url("/slow");
        let slow_context = context.clone();
        let slow = tokio::spawn(async move {
            background
                .browsing_context_navigate(NavigateParameters::new(
                    slow_context,
                    slow_url,
                    Some(ReadinessState::None),
                ))
                .await
        });
        let slow_url = server_url("/slow");
        let slow_navigation = started
            .next_matching(|e: &NavigationStarted| e.params.base.url == slow_url)
            .await?
            .params
            .base
            .navigation
            .ok_or_else(|| anyhow!("navigationStarted has no navigation id"))?;

        navigate(&mut session, &context, &inline("<p>replacement</p>")).await?;
        let is_slow = |navigation: &Option<Navigation>| navigation.as_ref() == Some(&slow_navigation);
        let reported_as_failed = match aborted.as_mut() {
            Some(aborted) => tokio::select! {
                event = aborted.next_matching(|e: &NavigationAborted| is_slow(&e.params.base.navigation)) => {
                    event?;
                    false
                }
                event = failed.next_matching(|e: &NavigationFailed| is_slow(&e.params.base.navigation)) => {
                    event?;
                    true
                }
            },
            None => {
                failed
                    .next_matching(|e: &NavigationFailed| is_slow(&e.params.base.navigation))
                    .await?;
                true
            }
        };
        if reported_as_failed {
            eprintln!("NOTE: the browser reported the interrupted navigation as navigationFailed");
        }
        // The interrupted command may fail or succeed; either is fine here.
        let _ = slow.await;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/create
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/close
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/context_destroyed
#[tokio::test]
async fn create_and_close_tab() -> Result<()> {
    with_session(|mut session| async move {
        let mut created = subscribe(&mut session, "browsingContext.contextCreated").await?;
        let mut destroyed = subscribe(&mut session, "browsingContext.contextDestroyed").await?;

        let tab = new_tab(&mut session).await?;
        created
            .next_matching(|event: &ContextCreated| event.params.info.context == tab)
            .await?;
        let tree = get_tree(&mut session, Some(0), None).await?;
        ensure!(
            tree.iter().any(|info| info.context == tab),
            "new tab missing from getTree"
        );

        session
            .browsing_context_close(CloseParameters::new(tab.clone(), None))
            .await?;
        destroyed
            .next_matching(|event: &ContextDestroyed| event.params.context == tab)
            .await?;
        let tree = get_tree(&mut session, Some(0), None).await?;
        ensure!(
            tree.iter().all(|info| info.context != tab),
            "closed tab still in getTree"
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn create_window_in_background() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let window = session
            .browsing_context_create(CreateParameters::new(
                CreateType::Window,
                Some(context.clone()),
                Some(true),
                None,
            ))
            .await?;
        let tree = get_tree(&mut session, Some(0), None).await?;
        let info = tree
            .iter()
            .find(|info| info.context == window.context)
            .ok_or_else(|| anyhow!("new window missing from getTree"))?;
        let original = tree
            .iter()
            .find(|info| info.context == context)
            .ok_or_else(|| anyhow!("original context missing"))?;
        ensure!(
            info.client_window != original.client_window,
            "a new window shares the original's client window"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/activate
#[tokio::test]
async fn activate_brings_context_to_front() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        new_tab(&mut session).await?;
        session
            .browsing_context_activate(ActivateParameters::new(context.clone()))
            .await?;
        wait_for(
            &mut session,
            &context,
            "document.visibilityState",
            "visible",
        )
        .await
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/get_tree
#[tokio::test]
async fn get_tree_includes_iframes() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let child_url = inline("<p>child</p>");
        navigate(
            &mut session,
            &context,
            &inline(&format!("<iframe src='{child_url}'></iframe>")),
        )
        .await?;

        let mut tree = get_tree(&mut session, None, Some(&context)).await?;
        let children = tree[0].children.take().unwrap_or_default();
        ensure!(
            children.len() == 1,
            "expected one iframe, got {}",
            children.len()
        );
        ensure!(
            children[0].url == child_url,
            "iframe url is {}",
            children[0].url
        );
        // `parent` is only set on the root of the returned tree.
        ensure!(
            tree[0].parent.is_none(),
            "top-level context has parent {:?}",
            tree[0].parent
        );

        let shallow = get_tree(&mut session, Some(0), Some(&context)).await?;
        ensure!(
            shallow[0].children.is_none(),
            "maxDepth 0 still returned children"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/locate_nodes
#[tokio::test]
async fn locate_nodes_with_every_locator() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let child_url = inline("<p>child</p>");
        navigate(
            &mut session,
            &context,
            &inline(&format!(
                "<div class='a'>one</div><div class='a'>Two</div><button>Go</button>\
                 <iframe src='{child_url}'></iframe>"
            )),
        )
        .await?;
        let child = get_tree(&mut session, None, Some(&context))
            .await?
            .swap_remove(0)
            .children
            .unwrap_or_default()
            .into_iter()
            .next()
            .map(|info| info.context)
            .ok_or_else(|| anyhow!("iframe context missing"))?;

        let cases: Vec<(&str, Locator, usize, &str)> = vec![
            (
                "css",
                Locator::CssLocator(CssLocator::new("div.a".into())),
                2,
                "div",
            ),
            (
                "xpath",
                Locator::XPathLocator(XPathLocator::new("//div".into())),
                2,
                "div",
            ),
            (
                "innerText",
                Locator::InnerTextLocator(InnerTextLocator::new(
                    "two".into(),
                    Some(true),
                    Some(InnerTextLocatorMatchType::Full),
                    None,
                )),
                1,
                "div",
            ),
            (
                "accessibility",
                Locator::AccessibilityLocator(AccessibilityLocator::new(
                    AccessibilityLocatorValue::new(Some("Go".into()), Some("button".into())),
                )),
                1,
                "button",
            ),
            (
                "context",
                Locator::ContextLocator(ContextLocator::new(ContextValue::new(child))),
                1,
                "iframe",
            ),
        ];
        for (name, locator, expected, local_name) in cases {
            let located = session
                .browsing_context_locate_nodes(LocateNodesParameters::new(
                    context.clone(),
                    locator,
                    None,
                    None,
                    None,
                ))
                .await;
            // Firefox 161 doesn't support the innerText locator yet.
            let Some(located) =
                supported(&format!("browsingContext.locateNodes ({name})"), located)?
            else {
                continue;
            };
            let nodes = located.nodes;
            ensure!(
                nodes.len() == expected,
                "{name}: {} nodes, expected {expected}",
                nodes.len()
            );
            let found = nodes[0].value.as_ref().and_then(|v| v.local_name.clone());
            ensure!(
                found.as_deref() == Some(local_name),
                "{name}: found <{found:?}>, expected <{local_name}>"
            );
        }

        let limited = session
            .browsing_context_locate_nodes(LocateNodesParameters::new(
                context.clone(),
                Locator::CssLocator(CssLocator::new("div".into())),
                Some(1),
                None,
                None,
            ))
            .await?
            .nodes;
        ensure!(
            limited.len() == 1,
            "maxNodeCount 1 returned {}",
            limited.len()
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
                context.clone(),
                None,
                None,
                None,
                None,
            ))
            .await?;
        // Base64 of the PNG signature.
        ensure!(
            screenshot.data.starts_with("iVBORw0KGgo"),
            "not a PNG: {}…",
            &screenshot.data[..screenshot.data.len().min(16)]
        );

        let jpeg = session
            .browsing_context_capture_screenshot(CaptureScreenshotParameters::new(
                context,
                Some(CaptureScreenshotParametersOrigin::Document),
                Some(ImageFormat::new("image/jpeg".into(), Some(0.5))),
                Some(ClipRectangle::BoxClipRectangle(BoxClipRectangle::new(
                    0.0, 0.0, 50.0, 50.0,
                ))),
                None,
            ))
            .await?;
        // Base64 of the JPEG SOI marker.
        ensure!(jpeg.data.starts_with("/9j/"), "not a JPEG");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/print
#[tokio::test]
async fn print_returns_pdf() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<h1>printable</h1>")).await?;
        let pdf = session
            .browsing_context_print(PrintParameters::new(
                context,
                Some(true),
                Some(PrintMarginParameters::new(
                    Some(1.0),
                    Some(1.0),
                    Some(1.0),
                    Some(1.0),
                )),
                Some(PrintParametersOrientation::Landscape),
                Some(PrintPageParameters::new(Some(21.0), Some(29.7))),
                Some(vec![JsUintOrText::JsUint(1)]),
                Some(1.0),
                Some(true),
            ))
            .await?;
        // Base64 of "%PDF-".
        ensure!(pdf.data.starts_with("JVBERi0"), "not a PDF");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/reload
#[tokio::test]
async fn reload_reloads_the_page() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let url = inline("<p>reload</p>");
        navigate(&mut session, &context, &url).await?;
        evaluate(&mut session, &context, "window.__marker = 1").await?;
        let reloaded = session
            .browsing_context_reload(ReloadParameters::new(
                context.clone(),
                Some(true),
                Some(ReadinessState::Complete),
            ))
            .await;
        // Firefox 161 doesn't support `ignoreCache` yet; reload without it.
        let result = match supported("browsingContext.reload with ignoreCache", reloaded)? {
            Some(result) => result,
            None => {
                session
                    .browsing_context_reload(ReloadParameters::new(
                        context.clone(),
                        None,
                        Some(ReadinessState::Complete),
                    ))
                    .await?
            }
        };
        ensure!(result.url == url, "reload returned {}", result.url);
        ensure!(
            eval_string(&mut session, &context, "String(window.__marker)").await? == "undefined",
            "page state survived the reload"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/set_viewport
#[tokio::test]
async fn set_viewport_changes_window_size() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        session
            .browsing_context_set_viewport(SetViewportParameters::new(
                Some(context.clone()),
                Some(Viewport::new(500, 400)),
                Some(2.0),
                None,
            ))
            .await?;
        let size = eval_string(
            &mut session,
            &context,
            "`${innerWidth}x${innerHeight}@${devicePixelRatio}`",
        )
        .await?;
        ensure!(size == "500x400@2", "viewport is {size}");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/set_bypass_csp
#[tokio::test]
async fn set_bypass_csp_allows_blocked_scripts() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let url = inline(
            "<meta http-equiv='Content-Security-Policy' content=\"script-src 'none'\">\
             <script>window.__ran = 'yes'</script>",
        );
        navigate(&mut session, &context, &url).await?;
        ensure!(
            eval_string(&mut session, &context, "String(window.__ran)").await? == "undefined",
            "CSP didn't block the inline script"
        );

        let bypass = session
            .browsing_context_set_bypass_csp(SetBypassCSPParameters::new(
                Some(true),
                Some(vec![context.clone()]),
                None,
            ))
            .await;
        if supported("browsingContext.setBypassCSP", bypass)?.is_none() {
            return Ok(());
        }
        navigate(&mut session, &context, &url).await?;
        ensure!(
            eval_string(&mut session, &context, "String(window.__ran)").await? == "yes",
            "the inline script is still blocked"
        );

        session
            .browsing_context_set_bypass_csp(SetBypassCSPParameters::new(
                None,
                Some(vec![context.clone()]),
                None,
            ))
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/traverse_history
#[tokio::test]
async fn traverse_history_goes_back_and_forward() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let first = inline("<p>first</p>");
        let second = inline("<p>second</p>");
        navigate(&mut session, &context, &first).await?;
        navigate(&mut session, &context, &second).await?;

        session
            .browsing_context_traverse_history(TraverseHistoryParameters::new(context.clone(), -1))
            .await?;
        wait_for(&mut session, &context, "location.href", &first).await?;
        session
            .browsing_context_traverse_history(TraverseHistoryParameters::new(context.clone(), 1))
            .await?;
        wait_for(&mut session, &context, "location.href", &second).await
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/handle_user_prompt
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/user_prompt_opened
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/user_prompt_closed
#[tokio::test]
async fn handle_user_prompt_with_events() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<p>prompts</p>")).await?;
        let mut opened = subscribe(&mut session, "browsingContext.userPromptOpened").await?;
        let mut closed = subscribe(&mut session, "browsingContext.userPromptClosed").await?;

        evaluate(
            &mut session,
            &context,
            "setTimeout(() => { window.__answer = prompt('Name?', 'default'); }, 0)",
        )
        .await?;
        let event: UserPromptOpened = opened.next().await?;
        ensure!(
            matches!(event.params.prompt_type, UserPromptType::Prompt),
            "prompt type is {:?}",
            event.params.prompt_type
        );
        ensure!(
            event.params.message == "Name?",
            "message is {}",
            event.params.message
        );
        ensure!(
            event.params.default_value.as_deref() == Some("default"),
            "default value is {:?}",
            event.params.default_value
        );

        session
            .browsing_context_handle_user_prompt(HandleUserPromptParameters::new(
                context.clone(),
                Some(true),
                Some("Rust".into()),
            ))
            .await?;
        let event: UserPromptClosed = closed.next().await?;
        ensure!(event.params.accepted, "prompt was not accepted");
        ensure!(
            event.params.user_text.as_deref() == Some("Rust"),
            "user text is {:?}",
            event.params.user_text
        );
        wait_for(&mut session, &context, "String(window.__answer)", "Rust").await
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/download_will_begin
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context/download_end
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/set_download_behavior
#[tokio::test]
async fn download_events_and_behavior() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let folder = temp_dir("downloads")?;
        let allowed = session
            .browser_set_download_behavior(SetDownloadBehaviorParameters::new(
                Some(DownloadBehavior::Allowed {
                    destination_folder: folder.to_string_lossy().into(),
                }),
                None,
            ))
            .await;
        if supported("browser.setDownloadBehavior", allowed)?.is_none() {
            return Ok(());
        }
        let mut will_begin = subscribe(&mut session, "browsingContext.downloadWillBegin").await?;
        let mut ended = subscribe(&mut session, "browsingContext.downloadEnd").await?;
        navigate(
            &mut session,
            &context,
            &inline(&format!(
                "<a href='{}' download>get</a>",
                server_url("/download")
            )),
        )
        .await?;
        evaluate(
            &mut session,
            &context,
            "document.querySelector('a').click()",
        )
        .await?;

        let begin: DownloadWillBegin = will_begin.next().await?;
        ensure!(
            begin.params.suggested_filename == "file.txt",
            "suggested filename is {}",
            begin.params.suggested_filename
        );
        let end: DownloadEnd = ended.next().await?;
        match end.params {
            DownloadEndParams::DownloadComplete(complete) => {
                let path = complete
                    .filepath
                    .ok_or_else(|| anyhow!("completed download has no filepath"))?;
                let content = std::fs::read_to_string(&path)?;
                ensure!(content == "downloaded content", "downloaded {content:?}");
            }
            DownloadEndParams::DownloadCanceled(_) => bail!("download was canceled"),
        }

        session
            .browser_set_download_behavior(SetDownloadBehaviorParameters::new(None, None))
            .await?;
        let _ = std::fs::remove_dir_all(folder);
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
