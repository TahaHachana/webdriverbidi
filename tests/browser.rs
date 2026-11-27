//! Live tests for the `browser` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use anyhow::{Result, anyhow, ensure};
use webdriverbidi::model::browser::*;
use webdriverbidi::model::browsing_context::{
    ContextDestroyed, CreateParameters, CreateType, GetTreeParameters,
};
use webdriverbidi::model::common::{EmptyParams, Extensible};
use webdriverbidi::model::session::{DirectProxyConfiguration, ProxyConfiguration};
use webdriverbidi::session::WebDriverBiDiSession;

use support::{
    eval_string, evaluate, inline, navigate, new_session, subscribe, supported, with_session,
};

async fn user_context_ids(session: &mut WebDriverBiDiSession) -> Result<Vec<UserContext>> {
    Ok(session
        .browser_get_user_contexts(EmptyParams::new())
        .await?
        .user_contexts
        .into_iter()
        .map(|info| info.user_context)
        .collect())
}

async fn tab_in(session: &mut WebDriverBiDiSession, user_context: &str) -> Result<String> {
    Ok(session
        .browsing_context_create(CreateParameters::new(
            CreateType::Tab,
            None,
            None,
            Some(user_context.to_string()),
        ))
        .await?
        .context)
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/create_user_context
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/get_user_contexts
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/remove_user_context
#[tokio::test]
async fn create_list_and_remove_user_contexts() -> Result<()> {
    with_session(|mut session| async move {
        ensure!(
            user_context_ids(&mut session)
                .await?
                .contains(&"default".to_string()),
            "the default user context is missing"
        );
        let first = session
            .browser_create_user_context(CreateUserContextParameters::new(Some(true), None, None))
            .await?
            .user_context;
        let second = session
            .browser_create_user_context(CreateUserContextParameters::new(Some(false), None, None))
            .await?
            .user_context;
        ensure!(first != second, "user context ids are not unique");
        let ids = user_context_ids(&mut session).await?;
        ensure!(
            ids.contains(&first) && ids.contains(&second),
            "created contexts not listed"
        );

        session
            .browser_remove_user_context(RemoveUserContextParameters::new(first.clone()))
            .await?;
        let ids = user_context_ids(&mut session).await?;
        ensure!(!ids.contains(&first), "removed context still listed");
        ensure!(ids.contains(&second), "the other context was removed too");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn create_user_context_with_proxy() -> Result<()> {
    with_session(|mut session| async move {
        let proxy = ProxyConfiguration::DirectProxyConfiguration(DirectProxyConfiguration {
            proxy_type: "direct".into(),
            extensible: Extensible::new(),
        });
        let created = session
            .browser_create_user_context(CreateUserContextParameters::new(None, Some(proxy), None))
            .await;
        if let Some(created) = supported("browser.createUserContext with proxy", created)? {
            let tab = tab_in(&mut session, &created.user_context).await?;
            navigate(&mut session, &tab, &inline("<p>direct</p>")).await?;
        }
        Ok(())
    })
    .await
}

#[tokio::test]
async fn removing_a_user_context_closes_its_tabs() -> Result<()> {
    with_session(|mut session| async move {
        let user_context = session
            .browser_create_user_context(CreateUserContextParameters::new(None, None, None))
            .await?
            .user_context;
        let tab = tab_in(&mut session, &user_context).await?;
        let mut destroyed = subscribe(&mut session, "browsingContext.contextDestroyed").await?;

        session
            .browser_remove_user_context(RemoveUserContextParameters::new(user_context))
            .await?;
        destroyed
            .next_matching(|e: &ContextDestroyed| e.params.context == tab)
            .await?;
        let tree = session
            .browsing_context_get_tree(GetTreeParameters::new(Some(0), None))
            .await?;
        ensure!(
            tree.contexts.iter().all(|info| info.context != tab),
            "the tab outlived its user context"
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn user_contexts_isolate_storage() -> Result<()> {
    with_session(|mut session| async move {
        let url = inline("<p>storage</p>");
        let mut tabs = Vec::new();
        for _ in 0..2 {
            let user_context = session
                .browser_create_user_context(CreateUserContextParameters::new(None, None, None))
                .await?
                .user_context;
            let tab = tab_in(&mut session, &user_context).await?;
            navigate(&mut session, &tab, &url).await?;
            tabs.push(tab);
        }
        evaluate(
            &mut session,
            &tabs[0],
            "localStorage.setItem('key', 'first')",
        )
        .await?;
        let seen = eval_string(
            &mut session,
            &tabs[1],
            "String(localStorage.getItem('key'))",
        )
        .await?;
        ensure!(
            seen == "null",
            "storage leaked between user contexts: {seen}"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/get_client_windows
#[tokio::test]
async fn get_client_windows_tracks_new_windows() -> Result<()> {
    with_session(|mut session| async move {
        let before = session
            .browser_get_client_windows(EmptyParams::new())
            .await?
            .client_windows;
        ensure!(!before.is_empty(), "no client windows");
        ensure!(
            before.iter().all(|w| w.width > 0 && w.height > 0),
            "zero-sized window"
        );

        session
            .browsing_context_create(CreateParameters::new(CreateType::Window, None, None, None))
            .await?;
        let after = session
            .browser_get_client_windows(EmptyParams::new())
            .await?
            .client_windows;
        ensure!(
            after.len() == before.len() + 1,
            "{} windows before, {} after creating one",
            before.len(),
            after.len()
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/set_client_window_state
#[tokio::test]
async fn set_client_window_state() -> Result<()> {
    with_session(|mut session| async move {
        let window = session
            .browser_get_client_windows(EmptyParams::new())
            .await?
            .client_windows
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("no client window"))?;

        let resized = session
            .browser_set_client_window_state(SetClientWindowStateParameters::new(
                window.client_window.clone(),
                ClientWindowNamedOrRectState::ClientWindowRectState(ClientWindowRectState::new(
                    "normal".into(),
                    Some(640),
                    Some(480),
                    Some(10),
                    Some(20),
                )),
            ))
            .await;
        let Some(resized) = supported("browser.setClientWindowState", resized)? else {
            return Ok(());
        };
        ensure!(
            resized.client_window == window.client_window,
            "a different window changed"
        );
        ensure!(
            (resized.width, resized.height) == (640, 480),
            "window is {}x{}",
            resized.width,
            resized.height
        );

        let named = session
            .browser_set_client_window_state(SetClientWindowStateParameters::new(
                window.client_window,
                ClientWindowNamedOrRectState::ClientWindowNamedState(ClientWindowNamedState::new(
                    ClientWindowState::Normal,
                )),
            ))
            .await?;
        ensure!(
            named.state == ClientWindowState::Normal,
            "state is {:?}",
            named.state
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser/close
#[tokio::test]
async fn browser_close_ends_the_browser() -> Result<()> {
    // Not run through `with_session`: closing the browser ends the session.
    let mut session = new_session().await?;
    let closed = session.browser_close(EmptyParams::new()).await;
    // Firefox only allows `browser.close` in sessions started over BiDi; end the
    // session over HTTP instead, so the next test can start one.
    if supported("browser.close", closed)?.is_none() {
        session.close().await?;
    }
    Ok(())
}
