//! Live error-path tests: the errors users hit in practice, per module. Needs a
//! WebDriver BiDi server; see `tests/support`.
//!
//! Each case must fail with the spec's error code. The traffic check in
//! `with_session` already proves the command the crate sent is spec-valid and
//! that the error response parses, so a different code is the browser's
//! deviation: it's printed, and fails the test only with `WEBDRIVERBIDI_STRICT_SPEC=1`.

mod support;

use anyhow::{Result, bail};
use webdriverbidi::error::CommandError;
use webdriverbidi::model::browser::*;
use webdriverbidi::model::browsing_context::*;
use webdriverbidi::model::common::{EmptyParams, Extensible};
use webdriverbidi::model::emulation::*;
use webdriverbidi::model::input::*;
use webdriverbidi::model::network::*;
use webdriverbidi::model::script::*;
use webdriverbidi::model::session::{
    SubscribeParameters, UnsubscribeByIDRequest, UnsubscribeParameters,
};
use webdriverbidi::model::storage::*;
use webdriverbidi::model::web_extension::*;

use support::{error_code, inline, navigate, top_context, with_session};

const MISSING: &str = "nonexistent-id";

/// Collects `(what, expected code, result)` and reports them together.
#[derive(Default)]
struct Expectations {
    failures: Vec<String>,
    deviations: Vec<String>,
}

impl Expectations {
    fn expect<T: std::fmt::Debug>(
        &mut self,
        what: &str,
        expected: &str,
        result: Result<T, CommandError>,
    ) {
        match result {
            Ok(value) => self.failures.push(format!(
                "{what}: succeeded with {value:?}, expected `{expected}`"
            )),
            Err(e) => match error_code(&e) {
                Some(code) if code == expected => {}
                Some(code) => self.deviations.push(format!(
                    "{what}: browser answered `{code}`, spec says `{expected}`"
                )),
                None => self
                    .failures
                    .push(format!("{what}: not an error response: {e}")),
            },
        }
    }

    fn finish(self) -> Result<()> {
        for deviation in &self.deviations {
            eprintln!("SPEC DEVIATION (browser): {deviation}");
        }
        let strict = std::env::var("WEBDRIVERBIDI_STRICT_SPEC").as_deref() == Ok("1");
        let mut failures = self.failures;
        if strict {
            failures.extend(self.deviations);
        }
        if !failures.is_empty() {
            bail!("{}", failures.join("\n"));
        }
        Ok(())
    }
}

fn bogus_ref() -> SharedReference {
    SharedReference {
        shared_id: MISSING.into(),
        handle: None,
        extensible: Extensible::new(),
    }
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browsing_context
#[tokio::test]
async fn browsing_context_errors() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<p>errors</p>")).await?;
        let mut e = Expectations::default();

        e.expect(
            "navigate in a missing context",
            "no such frame",
            session
                .browsing_context_navigate(NavigateParameters::new(
                    MISSING.into(),
                    inline("<p>x</p>"),
                    None,
                ))
                .await,
        );
        e.expect(
            "getTree from a missing root",
            "no such frame",
            session
                .browsing_context_get_tree(GetTreeParameters::new(None, Some(MISSING.into())))
                .await,
        );
        e.expect(
            "handleUserPrompt without a prompt",
            "no such alert",
            session
                .browsing_context_handle_user_prompt(HandleUserPromptParameters::new(
                    context.clone(),
                    Some(true),
                    None,
                ))
                .await,
        );
        e.expect(
            "locateNodes with an invalid CSS selector",
            "invalid selector",
            session
                .browsing_context_locate_nodes(LocateNodesParameters::new(
                    context.clone(),
                    Locator::CssLocator(CssLocator::new("div[".into())),
                    None,
                    None,
                    None,
                ))
                .await,
        );
        e.expect(
            "traverseHistory past the first entry",
            "no such history entry",
            session
                .browsing_context_traverse_history(TraverseHistoryParameters::new(
                    context.clone(),
                    -100,
                ))
                .await,
        );
        e.expect(
            "create a tab in a missing user context",
            "no such user context",
            session
                .browsing_context_create(CreateParameters::new(
                    CreateType::Tab,
                    None,
                    None,
                    Some(MISSING.into()),
                ))
                .await,
        );
        e.finish()
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/script
#[tokio::test]
async fn script_errors() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let mut e = Expectations::default();

        e.expect(
            "evaluate in a missing realm",
            "no such frame",
            session
                .script_evaluate(EvaluateParameters::new(
                    "1".into(),
                    Target::RealmTarget(RealmTarget::new(MISSING.into())),
                    false,
                    None,
                    None,
                    None,
                ))
                .await,
        );
        e.expect(
            "callFunction with an unknown node reference",
            "no such node",
            session
                .script_call_function(CallFunctionParameters::new(
                    "n => n".into(),
                    false,
                    Target::ContextTarget(ContextTarget::new(context.clone(), None)),
                    Some(vec![LocalValue::RemoteReference(
                        RemoteReference::SharedReference(bogus_ref()),
                    )]),
                    None,
                    None,
                    None,
                    None,
                ))
                .await,
        );
        e.expect(
            "removePreloadScript with an unknown id",
            "no such script",
            session
                .script_remove_preload_script(RemovePreloadScriptParameters::new(MISSING.into()))
                .await,
        );
        e.finish()
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/browser
#[tokio::test]
async fn browser_errors() -> Result<()> {
    with_session(|mut session| async move {
        let mut e = Expectations::default();
        e.expect(
            "removeUserContext of the default context",
            "invalid argument",
            session
                .browser_remove_user_context(RemoveUserContextParameters::new("default".into()))
                .await,
        );
        e.expect(
            "removeUserContext of a missing context",
            "no such user context",
            session
                .browser_remove_user_context(RemoveUserContextParameters::new(MISSING.into()))
                .await,
        );
        e.expect(
            "setClientWindowState of a missing window",
            "no such client window",
            session
                .browser_set_client_window_state(SetClientWindowStateParameters::new(
                    MISSING.into(),
                    ClientWindowNamedOrRectState::ClientWindowNamedState(
                        ClientWindowNamedState::new(ClientWindowState::Normal),
                    ),
                ))
                .await,
        );
        e.finish()
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/network
#[tokio::test]
async fn network_errors() -> Result<()> {
    with_session(|mut session| async move {
        let mut e = Expectations::default();
        e.expect(
            "removeIntercept with an unknown id",
            "no such intercept",
            session
                .network_remove_intercept(RemoveInterceptParameters::new(MISSING.into()))
                .await,
        );
        e.expect(
            "continueRequest for an unknown request",
            "no such request",
            session
                .network_continue_request(ContinueRequestParameters::new(
                    MISSING.into(),
                    None,
                    None,
                    None,
                    None,
                    None,
                ))
                .await,
        );
        e.expect(
            "failRequest for an unknown request",
            "no such request",
            session
                .network_fail_request(FailRequestParameters::new(MISSING.into()))
                .await,
        );
        e.expect(
            "removeDataCollector with an unknown id",
            "no such network collector",
            session
                .network_remove_data_collector(RemoveDataCollectorParameters {
                    collector: MISSING.into(),
                })
                .await,
        );
        e.finish()
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/storage
#[tokio::test]
async fn storage_errors() -> Result<()> {
    with_session(|mut session| async move {
        let mut e = Expectations::default();
        e.expect(
            "getCookies in a missing context's partition",
            "no such frame",
            session
                .storage_get_cookies(GetCookiesParameters::new(
                    None,
                    Some(PartitionDescriptor::BrowsingContextPartitionDescriptor(
                        BrowsingContextPartitionDescriptor::new(MISSING.into()),
                    )),
                ))
                .await,
        );
        e.expect(
            "getCookies in a missing user context's partition",
            "no such user context",
            session
                .storage_get_cookies(GetCookiesParameters::new(
                    None,
                    Some(PartitionDescriptor::StorageKeyPartitionDescriptor(
                        StorageKeyPartitionDescriptor::new(Some(MISSING.into()), None),
                    )),
                ))
                .await,
        );
        e.finish()
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/input
#[tokio::test]
async fn input_errors() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<input id='text'>")).await?;
        let mut e = Expectations::default();
        e.expect(
            "performActions in a missing context",
            "no such frame",
            session
                .input_perform_actions(PerformActionsParameters::new(MISSING.into(), vec![]))
                .await,
        );
        e.expect(
            "pointer move relative to an unknown node",
            "no such node",
            session
                .input_perform_actions(PerformActionsParameters::new(
                    context.clone(),
                    vec![SourceActions::PointerSourceActions(
                        PointerSourceActions::new(
                            "mouse".into(),
                            None,
                            vec![PointerSourceAction::PointerMoveAction(
                                PointerMoveAction::new(
                                    0.0,
                                    0.0,
                                    None,
                                    Some(Origin::ElementOrigin(ElementOrigin::new(bogus_ref()))),
                                    PointerCommonProperties::new(
                                        None, None, None, None, None, None, None,
                                    ),
                                ),
                            )],
                        ),
                    )],
                ))
                .await,
        );
        let text_input =
            match support::evaluate(&mut session, &context, "document.getElementById('text')")
                .await?
            {
                RemoteValue::NodeRemoteValue(node) => SharedReference {
                    shared_id: node.shared_id.unwrap_or_default(),
                    handle: None,
                    extensible: Extensible::new(),
                },
                other => bail!("expected a node, got {other:?}"),
            };
        e.expect(
            "setFiles on a text input",
            "unable to set file input",
            session
                .input_set_files(SetFilesParameters::new(
                    context.clone(),
                    text_input,
                    vec!["/tmp/a.txt".into()],
                ))
                .await,
        );
        e.finish()
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/emulation
#[tokio::test]
async fn emulation_errors() -> Result<()> {
    with_session(|mut session| async move {
        let mut e = Expectations::default();
        e.expect(
            "override in a missing context",
            "no such frame",
            session
                .set_timezone_override(SetTimezoneOverrideParameters {
                    timezone: Some("UTC".into()),
                    contexts: Some(vec![MISSING.into()]),
                    user_contexts: None,
                })
                .await,
        );
        e.expect(
            "override in a missing user context",
            "no such user context",
            session
                .set_timezone_override(SetTimezoneOverrideParameters {
                    timezone: Some("UTC".into()),
                    contexts: None,
                    user_contexts: Some(vec![MISSING.into()]),
                })
                .await,
        );
        e.finish()
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/session
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/web_extension
#[tokio::test]
async fn session_and_web_extension_errors() -> Result<()> {
    with_session(|mut session| async move {
        let mut e = Expectations::default();
        e.expect(
            "subscribe to an unknown event",
            "invalid argument",
            session
                .session_subscribe(SubscribeParameters::new(
                    vec!["nonexistent.event".into()],
                    None,
                    None,
                ))
                .await,
        );
        e.expect(
            "unsubscribe an unknown subscription",
            "invalid argument",
            session
                .session_unsubscribe(UnsubscribeParameters::UnsubscribeByIDRequest(
                    UnsubscribeByIDRequest::new(vec![MISSING.into()]),
                ))
                .await,
        );
        e.expect(
            "uninstall an unknown extension",
            "no such web extension",
            session
                .web_extension_uninstall(UninstallParameters::new(MISSING.into()))
                .await,
        );
        // Make sure the session still works after a run of errors.
        session.session_status(EmptyParams::new()).await?;
        e.finish()
    })
    .await
}
