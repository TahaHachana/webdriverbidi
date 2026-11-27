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

pub mod cddl;

use std::collections::HashMap;
use std::future::Future;
use std::panic::{AssertUnwindSafe, resume_unwind};
use std::str::FromStr;
use std::sync::{Arc, Mutex, OnceLock};
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
use webdriverbidi::model::script::{
    ContextTarget, EvaluateParameters, EvaluationResult, PrimitiveProtocolValue, RemoteValue,
    ResultOwnership, Target,
};
use webdriverbidi::model::session::SubscribeParameters;
use webdriverbidi::session::{MessageDirection, WebDriverBiDiSession};
use webdriverbidi::webdriver::capabilities::CapabilitiesRequest;

/// How long to wait for an expected event.
pub const EVENT_TIMEOUT: Duration = Duration::from_secs(10);

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

/// The browser under test, from `WEBDRIVERBIDI_BROWSER` (default `chrome`).
pub fn browser() -> String {
    env_or("WEBDRIVERBIDI_BROWSER", "chrome")
}

/// Whether the browser under test is Firefox, for recording known gaps.
pub fn is_firefox() -> bool {
    browser() == "firefox"
}

/// Capabilities for the configured browser, headless unless `WEBDRIVERBIDI_HEADED=1`.
///
/// User prompts are left open (`unhandledPromptBehavior: ignore`) so tests can
/// observe and handle them.
pub fn capabilities() -> CapabilitiesRequest {
    let mut caps = CapabilitiesRequest::default();
    caps.add_extension(
        "unhandledPromptBehavior".into(),
        json!({ "default": "ignore" }),
    );
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
        let driver = match env_or("WEBDRIVERBIDI_BROWSER", "chrome").as_str() {
            "firefox" => "geckodriver",
            _ => "chromedriver",
        };
        format!(
            "could not start a session on {host}:{port}; is a WebDriver BiDi server running, \
             without another session open? (scripts/install-{driver}.sh && drivers/{driver} --port={port})"
        )
    })?;
    Ok(session)
}

/// Run `test` with a fresh session, and end the session afterwards even if the
/// test fails or panics.
///
/// Every message of the session is recorded. When the test passes, the traffic is
/// checked with [`cddl::check_traffic`]: commands must match `remote.cddl`, and
/// everything received must parse into the crate's types without losing data.
/// Received messages that don't match `local.cddl` are browser deviations from
/// the spec; they are printed, and fail the test only with `WEBDRIVERBIDI_STRICT_SPEC=1`.
pub async fn with_session<F, Fut>(test: F) -> Result<()>
where
    F: FnOnce(WebDriverBiDiSession) -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let mut session = new_session().await?;
    let traffic: Arc<Mutex<Vec<(MessageDirection, Value)>>> = Arc::default();
    let recorder = traffic.clone();
    session.set_message_observer(Some(Arc::new(move |direction, message: &Value| {
        recorder.lock().unwrap().push((direction, message.clone()));
    })));

    let outcome = AssertUnwindSafe(test(session.clone())).catch_unwind().await;
    session.set_message_observer(None);
    let closed = session.close().await;
    match outcome {
        Err(panic) => resume_unwind(panic),
        Ok(result) => {
            result?;
            closed.context("ending the session")?;
            let traffic = std::mem::take(&mut *traffic.lock().unwrap());
            check_recorded_traffic(&traffic)
        }
    }
}

fn check_recorded_traffic(traffic: &[(MessageDirection, Value)]) -> Result<()> {
    let report = cddl::check_traffic(traffic);
    // The same deviation repeats for every similar message; print each once.
    let mut seen: Vec<(String, usize)> = Vec::new();
    for deviation in &report.spec_deviations {
        let summary = deviation.lines().next().unwrap_or_default().to_string();
        match seen.iter_mut().find(|(s, _)| *s == summary) {
            Some((_, count)) => *count += 1,
            None => {
                eprintln!("SPEC DEVIATION (browser): {deviation}");
                seen.push((summary, 1));
            }
        }
    }
    for (summary, count) in seen.iter().filter(|(_, count)| *count > 1) {
        eprintln!("  (seen {count} times: {summary})");
    }
    let strict = std::env::var("WEBDRIVERBIDI_STRICT_SPEC").as_deref() == Ok("1");
    let mut failures = report.crate_bugs.clone();
    if strict {
        failures.extend(report.spec_deviations.iter().cloned());
    }
    anyhow::ensure!(
        failures.is_empty(),
        "{} problem(s) in the session's {} messages:\n  {}",
        failures.len(),
        report.messages,
        failures.join("\n  ")
    );
    Ok(())
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

/// Evaluate `expression` in `context` (awaiting promises) and return the remote value.
pub async fn evaluate(
    session: &mut WebDriverBiDiSession,
    context: &str,
    expression: &str,
) -> Result<RemoteValue> {
    let result = session
        .script_evaluate(EvaluateParameters::new(
            expression.into(),
            Target::ContextTarget(ContextTarget::new(context.into(), None)),
            true,
            Some(ResultOwnership::Root),
            None,
            None,
        ))
        .await?;
    match result {
        EvaluationResult::EvaluationResultSuccess(success) => Ok(success.result),
        other => Err(anyhow!("`{expression}` threw: {other:?}")),
    }
}

/// Evaluate an expression that produces a string (e.g. `JSON.stringify(...)`).
pub async fn eval_string(
    session: &mut WebDriverBiDiSession,
    context: &str,
    expression: &str,
) -> Result<String> {
    match evaluate(session, context, expression).await? {
        RemoteValue::PrimitiveProtocolValue(PrimitiveProtocolValue::StringValue(s)) => Ok(s.value),
        other => Err(anyhow!("`{expression}` returned {other:?}, not a string")),
    }
}

/// Run `eval_string(expression)` on a clone of the session in a background task,
/// for page actions that block until the test reacts (intercepted requests,
/// auth challenges, prompts).
pub fn spawn_eval(
    session: &WebDriverBiDiSession,
    context: &str,
    expression: &str,
) -> tokio::task::JoinHandle<Result<String>> {
    let mut session = session.clone();
    let context = context.to_string();
    let expression = expression.to_string();
    tokio::spawn(async move { eval_string(&mut session, &context, &expression).await })
}

/// A fresh directory under the system temp dir, e.g. for downloads.
pub fn temp_dir(name: &str) -> Result<std::path::PathBuf> {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "webdriverbidi-tests-{}-{name}-{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
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

/// Like [`subscribe`], but `Ok(None)` (and a printed skip notice) if the browser
/// doesn't know the event.
pub async fn subscribe_if_supported(
    session: &mut WebDriverBiDiSession,
    method: &str,
) -> Result<Option<Events>> {
    match subscribe(session, method).await {
        Ok(events) => Ok(Some(events)),
        Err(e) => {
            let unknown = e
                .downcast_ref::<CommandError>()
                .is_some_and(|e| error_code(e) == Some("invalid argument"));
            if unknown {
                eprintln!("SKIP: the browser doesn't support the {method} event: {e}");
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
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

/// URL of a fixed route on the test server:
///
/// * `/auth`: `401` with a Basic challenge (realm `test`) until the request carries
///   credentials `user:pass`, then `200` with an HTML page.
/// * `/headers`: the request headers as a JSON object (lowercase names).
/// * `/download`: a small text file sent as an attachment named `file.txt`.
/// * `/cookie`: `200` that sets the cookie `served=yes`.
/// * `/slow`: an HTML page that takes 3 seconds to respond.
pub fn server_url(path: &str) -> String {
    format!("http://{}{path}", server().addr)
}

/// Host and port of the test server, e.g. for cookie domains.
pub fn server_host() -> String {
    server().addr.ip().to_string()
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
                use axum::http::{HeaderMap, header};
                use axum::response::{Html, IntoResponse};
                use axum::routing::get;
                let app = axum::Router::new()
                    .route(
                        "/inline/{id}",
                        get(move |Path(id): Path<usize>| async move {
                            match pages.lock().unwrap().get(&id) {
                                Some(html) => Html(html.clone()).into_response(),
                                None => StatusCode::NOT_FOUND.into_response(),
                            }
                        }),
                    )
                    .route(
                        "/auth",
                        get(|headers: HeaderMap| async move {
                            // base64("user:pass")
                            let authorized = headers
                                .get(header::AUTHORIZATION)
                                .is_some_and(|v| v == "Basic dXNlcjpwYXNz");
                            if authorized {
                                Html("<p>authenticated</p>").into_response()
                            } else {
                                (
                                    StatusCode::UNAUTHORIZED,
                                    [(header::WWW_AUTHENTICATE, "Basic realm=\"test\"")],
                                    "unauthorized",
                                )
                                    .into_response()
                            }
                        }),
                    )
                    .route(
                        "/headers",
                        get(|headers: HeaderMap| async move {
                            let map: serde_json::Map<String, Value> = headers
                                .iter()
                                .map(|(k, v)| {
                                    (k.as_str().to_string(), json!(v.to_str().unwrap_or("")))
                                })
                                .collect();
                            axum::Json(Value::Object(map))
                        }),
                    )
                    .route(
                        "/download",
                        get(|| async {
                            (
                                [
                                    (header::CONTENT_TYPE, "text/plain"),
                                    (
                                        header::CONTENT_DISPOSITION,
                                        "attachment; filename=\"file.txt\"",
                                    ),
                                ],
                                "downloaded content",
                            )
                        }),
                    )
                    .route(
                        "/slow",
                        get(|| async {
                            tokio::time::sleep(Duration::from_secs(3)).await;
                            Html("<p>slow</p>")
                        }),
                    )
                    .route(
                        "/cookie",
                        get(|| async {
                            (
                                [(header::SET_COOKIE, "served=yes; Path=/")],
                                Html("<p>cookie set</p>"),
                            )
                        }),
                    );
                let listener = tokio::net::TcpListener::from_std(listener).expect("tokio listener");
                axum::serve(listener, app).await.expect("test server");
            });
        });
        Server { addr, pages }
    })
}
