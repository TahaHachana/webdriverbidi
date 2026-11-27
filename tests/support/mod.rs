//! Shared harness for the live integration tests.
//!
//! The tests need a WebDriver BiDi server, e.g.:
//!
//! ```text
//! scripts/install-chromedriver.sh && drivers/chromedriver --port=4444
//! cargo test --tests -- --test-threads=1
//! ```
//!
//! Environment variables:
//!
//! * `WEBDRIVERBIDI_HOST` / `WEBDRIVERBIDI_PORT`: where the driver listens
//!   (default `localhost:4444`).
//! * `WEBDRIVERBIDI_BROWSER`: `chrome` (default) or `firefox`; selects the
//!   vendor options used to run headless.
//! * `WEBDRIVERBIDI_HEADED=1`: show the browser window, for debugging.
#![allow(dead_code)] // Each test crate uses a different subset of the helpers.

use std::collections::HashMap;
use std::future::Future;
use std::panic::{AssertUnwindSafe, resume_unwind};
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use futures::FutureExt;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tokio::sync::mpsc;

use webdriverbidi::error::CommandError;
use webdriverbidi::events::EventType;
use webdriverbidi::model::browsing_context::{
    BrowsingContext, GetTreeParameters, NavigateParameters, ReadinessState,
};
use webdriverbidi::model::session::SubscribeParameters;
use webdriverbidi::session::WebDriverBiDiSession;
use webdriverbidi::webdriver::capabilities::CapabilitiesRequest;

/// How long to wait for an expected event.
pub const EVENT_TIMEOUT: Duration = Duration::from_secs(10);

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

/// Capabilities for the configured browser, headless unless `WEBDRIVERBIDI_HEADED=1`.
pub fn capabilities() -> CapabilitiesRequest {
    let mut caps = CapabilitiesRequest::default();
    if std::env::var("WEBDRIVERBIDI_HEADED").as_deref() != Ok("1") {
        match env_or("WEBDRIVERBIDI_BROWSER", "chrome").as_str() {
            "firefox" => caps.add_extension(
                "moz:firefoxOptions".into(),
                json!({ "args": ["-headless"] }),
            ),
            _ => caps.add_extension(
                "goog:chromeOptions".into(),
                json!({ "args": ["--headless=new"] }),
            ),
        }
    }
    caps
}

/// Start a new session against the configured driver.
pub async fn new_session() -> Result<WebDriverBiDiSession> {
    let host = env_or("WEBDRIVERBIDI_HOST", "localhost");
    let port: u16 = env_or("WEBDRIVERBIDI_PORT", "4444")
        .parse()
        .context("WEBDRIVERBIDI_PORT must be a port number")?;
    let mut session = WebDriverBiDiSession::new(host.clone(), port, capabilities());
    session.start().await.with_context(|| {
        format!(
            "could not start a session on {host}:{port}; is a WebDriver BiDi server running? \
             (scripts/install-chromedriver.sh && drivers/chromedriver --port={port})"
        )
    })?;
    Ok(session)
}

/// Run `test` with a fresh session, and end the session afterwards even if the
/// test fails or panics.
pub async fn with_session<F, Fut>(test: F) -> Result<()>
where
    F: FnOnce(WebDriverBiDiSession) -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let mut session = new_session().await?;
    let outcome = AssertUnwindSafe(test(session.clone())).catch_unwind().await;
    let closed = session.close().await;
    match outcome {
        Err(panic) => resume_unwind(panic),
        Ok(result) => {
            result?;
            closed.context("ending the session")?;
            Ok(())
        }
    }
}

/// The first top-level browsing context of the session.
pub async fn top_context(session: &mut WebDriverBiDiSession) -> Result<BrowsingContext> {
    let tree = session
        .browsing_context_get_tree(GetTreeParameters::new(None, None))
        .await?;
    tree.contexts
        .first()
        .map(|info| info.context.clone())
        .ok_or_else(|| anyhow!("the session has no browsing context"))
}

/// Navigate `context` to `url` and wait for the page to load completely.
pub async fn navigate(session: &mut WebDriverBiDiSession, context: &str, url: &str) -> Result<()> {
    session
        .browsing_context_navigate(NavigateParameters::new(
            context.to_string(),
            url.to_string(),
            Some(ReadinessState::Complete),
        ))
        .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Unsupported commands
// ---------------------------------------------------------------------------

/// The spec error code of a failed command, e.g. `"no such frame"`.
pub fn error_code(error: &CommandError) -> Option<&str> {
    match error {
        CommandError::Error(json) => json.get("error").and_then(Value::as_str),
        _ => None,
    }
}

/// `Ok(None)` (and a printed skip notice) if the browser doesn't implement the
/// command yet, so the test passes until it does.
///
/// ```ignore
/// let Some(result) = supported("browsingContext.startScreencast", session.browsing_context_start_screencast(params).await)? else {
///     return Ok(());
/// };
/// ```
pub fn supported<T>(command: &str, result: Result<T, CommandError>) -> Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(e)
            if matches!(
                error_code(&e),
                Some("unsupported operation" | "unknown command")
            ) =>
        {
            eprintln!("SKIP: {command} is not supported by this browser: {e}");
            Ok(None)
        }
        Err(e) => Err(e).with_context(|| format!("{command} failed")),
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events of one type, received after [`subscribe`].
pub struct Events {
    method: String,
    rx: mpsc::UnboundedReceiver<Value>,
}

/// Subscribe to `method` (e.g. `"browsingContext.load"`) and collect its events.
///
/// The session keeps one handler per event type, so subscribing to the same event
/// twice replaces the first stream.
pub async fn subscribe(session: &mut WebDriverBiDiSession, method: &str) -> Result<Events> {
    let event_type = EventType::from_str(method)
        .map_err(|_| anyhow!("`{method}` is not an event this crate dispatches"))?;
    let (tx, rx) = mpsc::unbounded_channel();
    session
        .register_event_handler(event_type, move |event| {
            let tx = tx.clone();
            async move {
                let _ = tx.send(event);
            }
        })
        .await;
    session
        .session_subscribe(SubscribeParameters::new(
            vec![method.to_string()],
            None,
            None,
        ))
        .await?;
    Ok(Events {
        method: method.to_string(),
        rx,
    })
}

impl Events {
    /// The next event, parsed into `T` (the model's event type, e.g.
    /// `browsing_context::Load`).
    pub async fn next<T: DeserializeOwned>(&mut self) -> Result<T> {
        self.next_matching(|_: &T| true).await
    }

    /// The next event that parses into `T` and satisfies `predicate`. Events are
    /// dispatched on separate tasks, so don't rely on their order.
    pub async fn next_matching<T: DeserializeOwned>(
        &mut self,
        predicate: impl Fn(&T) -> bool,
    ) -> Result<T> {
        let deadline = tokio::time::Instant::now() + EVENT_TIMEOUT;
        loop {
            let raw = tokio::time::timeout_at(deadline, self.rx.recv())
                .await
                .map_err(|_| {
                    anyhow!(
                        "no matching `{}` event within {EVENT_TIMEOUT:?}",
                        self.method
                    )
                })?
                .ok_or_else(|| anyhow!("the `{}` event stream closed", self.method))?;
            let event: T = serde_json::from_value(raw.clone())
                .with_context(|| format!("parsing a `{}` event: {raw}", self.method))?;
            if predicate(&event) {
                return Ok(event);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Test pages
// ---------------------------------------------------------------------------

/// Serve `html` from a local HTTP server and return its URL, like WPT's `inline()`.
///
/// The server runs on its own thread, so it outlives each test's Tokio runtime.
pub fn inline(html: &str) -> String {
    let server = server();
    let mut pages = server.pages.lock().unwrap();
    let id = pages.len();
    pages.insert(id, html.to_string());
    format!("http://{}/inline/{id}", server.addr)
}

struct Server {
    addr: std::net::SocketAddr,
    pages: &'static Mutex<HashMap<usize, String>>,
}

fn server() -> &'static Server {
    static SERVER: OnceLock<Server> = OnceLock::new();
    SERVER.get_or_init(|| {
        let pages: &'static Mutex<HashMap<usize, String>> =
            Box::leak(Box::new(Mutex::new(HashMap::new())));
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind the test server");
        let addr = listener.local_addr().expect("test server address");
        listener
            .set_nonblocking(true)
            .expect("non-blocking listener");
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("test server runtime");
            runtime.block_on(async move {
                use axum::extract::Path;
                use axum::http::StatusCode;
                use axum::response::{Html, IntoResponse};
                let app = axum::Router::new().route(
                    "/inline/{id}",
                    axum::routing::get(move |Path(id): Path<usize>| async move {
                        match pages.lock().unwrap().get(&id) {
                            Some(html) => Html(html.clone()).into_response(),
                            None => StatusCode::NOT_FOUND.into_response(),
                        }
                    }),
                );
                let listener = tokio::net::TcpListener::from_std(listener).expect("tokio listener");
                axum::serve(listener, app).await.expect("test server");
            });
        });
        Server { addr, pages }
    })
}
