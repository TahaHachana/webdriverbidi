//! Live tests for the `network` module. Needs a WebDriver BiDi server; see `tests/support`.
//!
//! Intercepted requests block the page until the test reacts, so page actions run
//! on a cloned session in a background task (`spawn_eval`).

mod support;

use anyhow::{Result, anyhow, bail, ensure};
use webdriverbidi::model::network::*;
use webdriverbidi::session::WebDriverBiDiSession;

use support::{
    eval_string, inline, is_firefox, navigate, server_url, spawn_eval, subscribe, supported,
    top_context, with_session,
};

fn text(value: &str) -> BytesValue {
    BytesValue::StringValue(StringValue {
        value_type: "string".into(),
        value: value.into(),
    })
}

fn header(name: &str, value: &str) -> Header {
    Header::new(name.into(), text(value))
}

/// Intercept requests to `url` in `phase`; returns the intercept id.
async fn intercept(
    session: &mut WebDriverBiDiSession,
    phase: InterceptPhase,
    url: &str,
) -> Result<Intercept> {
    Ok(session
        .network_add_intercept(AddInterceptParameters::new(
            vec![phase],
            None,
            Some(vec![UrlPattern::UrlPatternString(UrlPatternString::new(
                url.into(),
            ))]),
        ))
        .await?
        .intercept)
}

/// A page on the test server, so `fetch` calls are same-origin.
async fn open_page(session: &mut WebDriverBiDiSession) -> Result<String> {
    let context = top_context(session).await?;
    navigate(session, &context, &inline("<p>network</p>")).await?;
    Ok(context)
}

fn fetch_text(url: &str) -> String {
    format!("fetch('{url}').then(r => r.text(), e => 'failed: ' + e.message)")
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/before_request_sent
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/response_started
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/response_completed
#[tokio::test]
async fn request_lifecycle_events() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let url = inline("<p>events</p>");
        let mut before = subscribe(&mut session, "network.beforeRequestSent").await?;
        let mut started = subscribe(&mut session, "network.responseStarted").await?;
        let mut completed = subscribe(&mut session, "network.responseCompleted").await?;

        navigate(&mut session, &context, &url).await?;

        let sent: BeforeRequestSent = before
            .next_matching(|e: &BeforeRequestSent| e.params.base.request.url == url)
            .await?;
        ensure!(
            sent.params.base.request.method == "GET",
            "method {}",
            sent.params.base.request.method
        );
        ensure!(
            !sent.params.base.is_blocked,
            "request blocked without an intercept"
        );
        started
            .next_matching(|e: &ResponseStarted| e.params.base.request.url == url)
            .await?;
        let done: ResponseCompleted = completed
            .next_matching(|e: &ResponseCompleted| e.params.base.request.url == url)
            .await?;
        ensure!(
            done.params.response.status == 200,
            "status {}",
            done.params.response.status
        );
        ensure!(
            done.params.base.request.request == sent.params.base.request.request,
            "events disagree on the request id"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/add_intercept
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/continue_request
#[tokio::test]
async fn continue_request_with_modified_headers() -> Result<()> {
    with_session(|mut session| async move {
        let context = open_page(&mut session).await?;
        let url = server_url("/headers");
        let id = intercept(&mut session, InterceptPhase::BeforeRequestSent, &url).await?;
        let mut before = subscribe(&mut session, "network.beforeRequestSent").await?;

        let page = spawn_eval(&session, &context, &fetch_text(&url));
        let blocked: BeforeRequestSent = before
            .next_matching(|e: &BeforeRequestSent| e.params.base.request.url == url)
            .await?;
        ensure!(blocked.params.base.is_blocked, "request was not blocked");
        ensure!(
            blocked
                .params
                .base
                .intercepts
                .as_deref()
                .unwrap_or_default()
                .contains(&id),
            "event doesn't list the intercept"
        );
        let mut headers = blocked.params.base.request.headers;
        headers.push(header("X-Intercepted", "yes"));
        session
            .network_continue_request(ContinueRequestParameters::new(
                blocked.params.base.request.request,
                None,
                None,
                Some(headers),
                None,
                None,
            ))
            .await?;

        let body = page.await??;
        let received: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| anyhow!("unexpected body {body:?}: {e}"))?;
        ensure!(received["x-intercepted"] == "yes", "server saw {received}");

        session
            .network_remove_intercept(RemoveInterceptParameters::new(id))
            .await?;
        let unblocked = eval_string(&mut session, &context, &fetch_text(&url)).await?;
        ensure!(
            !unblocked.contains("x-intercepted"),
            "the intercept outlived removeIntercept"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/fail_request
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/fetch_error
#[tokio::test]
async fn fail_request_fires_fetch_error() -> Result<()> {
    with_session(|mut session| async move {
        let context = open_page(&mut session).await?;
        let url = server_url("/headers");
        intercept(&mut session, InterceptPhase::BeforeRequestSent, &url).await?;
        let mut before = subscribe(&mut session, "network.beforeRequestSent").await?;
        let mut errors = subscribe(&mut session, "network.fetchError").await?;

        let page = spawn_eval(&session, &context, &fetch_text(&url));
        let blocked: BeforeRequestSent = before
            .next_matching(|e: &BeforeRequestSent| e.params.base.request.url == url)
            .await?;
        session
            .network_fail_request(FailRequestParameters::new(
                blocked.params.base.request.request,
            ))
            .await?;

        let error: FetchError = errors
            .next_matching(|e: &FetchError| e.params.base.request.url == url)
            .await?;
        ensure!(!error.params.error_text.is_empty(), "empty errorText");
        let body = page.await??;
        ensure!(body.starts_with("failed"), "fetch didn't fail: {body}");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/provide_response
#[tokio::test]
async fn provide_response_replaces_the_response() -> Result<()> {
    with_session(|mut session| async move {
        let context = open_page(&mut session).await?;
        let url = server_url("/headers");
        intercept(&mut session, InterceptPhase::BeforeRequestSent, &url).await?;
        let mut before = subscribe(&mut session, "network.beforeRequestSent").await?;

        let page = spawn_eval(&session, &context, &fetch_text(&url));
        let blocked: BeforeRequestSent = before
            .next_matching(|e: &BeforeRequestSent| e.params.base.request.url == url)
            .await?;
        session
            .network_provide_response(ProvideResponseParameters::new(
                blocked.params.base.request.request,
                Some(text("provided body")),
                None,
                Some(vec![header("Content-Type", "text/plain")]),
                Some("OK".into()),
                Some(200),
            ))
            .await?;
        let body = page.await??;
        ensure!(body == "provided body", "page received {body:?}");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/continue_response
#[tokio::test]
async fn continue_response_changes_the_status() -> Result<()> {
    with_session(|mut session| async move {
        let context = open_page(&mut session).await?;
        let url = server_url("/headers");
        intercept(&mut session, InterceptPhase::ResponseStarted, &url).await?;
        let mut started = subscribe(&mut session, "network.responseStarted").await?;

        let page = spawn_eval(
            &session,
            &context,
            &format!("fetch('{url}').then(r => `${{r.status}} ${{r.statusText}}`)"),
        );
        let blocked: ResponseStarted = started
            .next_matching(|e: &ResponseStarted| e.params.base.request.url == url)
            .await?;
        ensure!(blocked.params.base.is_blocked, "response was not blocked");
        ensure!(
            blocked.params.response.status == 200,
            "status {}",
            blocked.params.response.status
        );
        session
            .network_continue_response(ContinueResponseParameters::new(
                blocked.params.base.request.request,
                None,
                None,
                None,
                Some("Accepted".into()),
                Some(202),
            ))
            .await?;
        let status = page.await??;
        ensure!(status == "202 Accepted", "page saw {status}");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/auth_required
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/continue_with_auth
#[tokio::test]
async fn continue_with_auth_provides_credentials() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let url = server_url("/auth");
        intercept(&mut session, InterceptPhase::AuthRequired, &url).await?;
        let mut auth = subscribe(&mut session, "network.authRequired").await?;

        let mut navigator = session.clone();
        let nav_context = context.clone();
        let nav_url = url.clone();
        let page =
            tokio::spawn(async move { navigate(&mut navigator, &nav_context, &nav_url).await });

        let challenge: AuthRequired = auth
            .next_matching(|e: &AuthRequired| e.params.base.request.url == url)
            .await?;
        // Chrome 158 sends -1 here, which the crate reads as 0 ("no status").
        let status = challenge.params.response.status;
        ensure!(
            status == 401 || (!is_firefox() && status == 0),
            "status {status}"
        );
        let challenges = challenge
            .params
            .response
            .auth_challenges
            .unwrap_or_default();
        let has_basic = challenges
            .iter()
            .any(|c| c.scheme.eq_ignore_ascii_case("basic") && c.realm == "test");
        if is_firefox() {
            ensure!(has_basic, "auth challenges: {challenges:?}");
        } else if !has_basic {
            // Chrome 158 leaves `authChallenges` empty.
            eprintln!(
                "NOTE: known Chrome gap, authRequired without authChallenges: {challenges:?}"
            );
        }
        session
            .network_continue_with_auth(ContinueWithAuthParameters::new(
                challenge.params.base.request.request,
                Some(ContinueWithAuthOption::Credentials(
                    ContinueWithAuthCredentials::new(
                        "provideCredentials".into(),
                        AuthCredentials::new("user".into(), "pass".into()),
                    ),
                )),
            ))
            .await?;
        page.await??;
        let body = eval_string(&mut session, &context, "document.body.textContent").await?;
        ensure!(body == "authenticated", "page shows {body:?}");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn continue_with_auth_can_cancel() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let url = server_url("/auth");
        intercept(&mut session, InterceptPhase::AuthRequired, &url).await?;
        let mut auth = subscribe(&mut session, "network.authRequired").await?;

        let mut navigator = session.clone();
        let nav_context = context.clone();
        let nav_url = url.clone();
        let page =
            tokio::spawn(async move { navigate(&mut navigator, &nav_context, &nav_url).await });

        let challenge: AuthRequired = auth
            .next_matching(|e: &AuthRequired| e.params.base.request.url == url)
            .await?;
        session
            .network_continue_with_auth(ContinueWithAuthParameters::new(
                challenge.params.base.request.request,
                Some(ContinueWithAuthOption::NoCredentials(
                    ContinueWithAuthNoCredentials::new(NoCredentialsAction::Cancel),
                )),
            ))
            .await?;
        // Canceling shows the 401 response; the navigation itself may succeed or fail.
        let _ = page.await?;
        let body = eval_string(&mut session, &context, "document.body.textContent").await?;
        ensure!(body != "authenticated", "canceled auth still authenticated");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/set_extra_headers
#[tokio::test]
async fn set_extra_headers_adds_headers() -> Result<()> {
    with_session(|mut session| async move {
        let context = open_page(&mut session).await?;
        let set = session
            .network_set_extra_headers(SetExtraHeadersParameters {
                headers: vec![header("X-Extra", "added")],
                contexts: Some(vec![context.clone()]),
                user_contexts: None,
            })
            .await;
        if supported("network.setExtraHeaders", set)?.is_none() {
            return Ok(());
        }
        let body =
            eval_string(&mut session, &context, &fetch_text(&server_url("/headers"))).await?;
        let received: serde_json::Value = serde_json::from_str(&body)?;
        ensure!(received["x-extra"] == "added", "server saw {received}");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/set_cache_behavior
#[tokio::test]
async fn set_cache_behavior() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        session
            .network_set_cache_behavior(SetCacheBehaviorParameters::new(
                CacheBehavior::Bypass,
                Some(vec![context.clone()]),
            ))
            .await?;
        navigate(&mut session, &context, &inline("<p>uncached</p>")).await?;
        session
            .network_set_cache_behavior(SetCacheBehaviorParameters::new(
                CacheBehavior::Default,
                None,
            ))
            .await?;
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/add_data_collector
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network/get_data
#[tokio::test]
async fn data_collectors_capture_response_bodies() -> Result<()> {
    with_session(|mut session| async move {
        let context = open_page(&mut session).await?;
        let added = session
            .network_add_data_collector(AddDataCollectorParameters {
                data_types: vec!["response".into()],
                max_encoded_data_size: 100_000,
                collector_type: None,
                contexts: Some(vec![context.clone()]),
                user_contexts: None,
            })
            .await;
        let Some(added) = supported("network.addDataCollector", added)? else {
            return Ok(());
        };
        let mut completed = subscribe(&mut session, "network.responseCompleted").await?;
        let url = server_url("/download");
        eval_string(&mut session, &context, &fetch_text(&url)).await?;
        let done: ResponseCompleted = completed
            .next_matching(|e: &ResponseCompleted| e.params.base.request.url == url)
            .await?;
        let request = done.params.base.request.request;

        let data = session
            .network_get_data(GetDataParameters {
                data_type: "response".into(),
                collector: Some(added.collector.clone()),
                disown: None,
                request: request.clone(),
            })
            .await?;
        let body = match data.bytes {
            BytesValue::StringValue(s) => s.value,
            BytesValue::Base64Value(b) => {
                bail!("expected a text body, got base64 {}", b.value)
            }
        };
        ensure!(body == "downloaded content", "collected {body:?}");

        session
            .network_disown_data(DisownDataParameters {
                data_type: "response".into(),
                collector: added.collector.clone(),
                request,
            })
            .await?;
        session
            .network_remove_data_collector(RemoveDataCollectorParameters {
                collector: added.collector,
            })
            .await?;
        Ok(())
    })
    .await
}
