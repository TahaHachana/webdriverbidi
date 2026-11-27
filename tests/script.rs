//! Live tests for the `script` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use anyhow::{Result, anyhow, bail, ensure};
use webdriverbidi::model::common::Extensible;
use webdriverbidi::model::script::*;
use webdriverbidi::session::WebDriverBiDiSession;

use support::{
    error_code, eval_string, evaluate, inline, navigate, subscribe, top_context, with_session,
};

fn context_target(context: &str) -> Target {
    Target::ContextTarget(ContextTarget::new(context.into(), None))
}

fn string(v: &str) -> LocalValue {
    LocalValue::PrimitiveProtocolValue(PrimitiveProtocolValue::StringValue(StringValue::new(
        v.into(),
    )))
}

fn number(v: f64) -> LocalValue {
    LocalValue::PrimitiveProtocolValue(PrimitiveProtocolValue::NumberValue(NumberValue {
        value_type: "number".into(),
        value: NumberOrSpecialNumber::Number(v),
    }))
}

async fn call(
    session: &mut WebDriverBiDiSession,
    context: &str,
    function: &str,
    arguments: Vec<LocalValue>,
    this: Option<LocalValue>,
) -> Result<RemoteValue> {
    let result = session
        .script_call_function(CallFunctionParameters::new(
            function.into(),
            true,
            context_target(context),
            Some(arguments),
            Some(ResultOwnership::Root),
            None,
            this,
            None,
        ))
        .await?;
    match result {
        EvaluationResult::EvaluationResultSuccess(success) => Ok(success.result),
        other => bail!("`{function}` threw: {other:?}"),
    }
}

fn as_string(value: RemoteValue) -> Result<String> {
    match value {
        RemoteValue::PrimitiveProtocolValue(PrimitiveProtocolValue::StringValue(s)) => Ok(s.value),
        other => bail!("expected a string, got {other:?}"),
    }
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/script/evaluate
// Regression test: non-primitive remote values used to parse as `undefined`
// and lose their handle, value and sharedId.
#[tokio::test]
async fn evaluate_keeps_remote_value_data() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<div id='main'>hi</div>")).await?;

        match evaluate(&mut session, &context, "({ answer: 42 })").await? {
            RemoteValue::ObjectRemoteValue(object) => {
                ensure!(object.handle.is_some(), "object has no handle");
                let entries = object.value.unwrap_or_default();
                ensure!(
                    entries
                        .iter()
                        .any(|(key, _)| matches!(key, RemoteValueOrText::Text(k) if k == "answer")),
                    "object entries lost: {entries:?}"
                );
            }
            other => bail!("expected an object, got {other:?}"),
        }

        match evaluate(&mut session, &context, "document.getElementById('main')").await? {
            RemoteValue::NodeRemoteValue(node) => {
                ensure!(node.shared_id.is_some(), "node has no sharedId");
                let properties = node
                    .value
                    .ok_or_else(|| anyhow!("node has no properties"))?;
                ensure!(
                    properties.local_name.as_deref() == Some("div"),
                    "localName is {:?}",
                    properties.local_name
                );
            }
            other => bail!("expected a node, got {other:?}"),
        }

        match evaluate(&mut session, &context, "[1, 'two']").await? {
            RemoteValue::ArrayRemoteValue(array) => {
                ensure!(array.value.map(|v| v.len()) == Some(2), "array items lost");
            }
            other => bail!("expected an array, got {other:?}"),
        }
        Ok(())
    })
    .await
}

#[tokio::test]
async fn evaluate_reports_exceptions() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let result = session
            .script_evaluate(EvaluateParameters::new(
                "notDefined".into(),
                context_target(&context),
                false,
                None,
                None,
                None,
            ))
            .await?;
        match result {
            EvaluationResult::EvaluationResultException(exception) => {
                ensure!(
                    exception.exception_details.text.contains("notDefined"),
                    "unexpected exception text: {}",
                    exception.exception_details.text
                );
                Ok(())
            }
            other => bail!("expected an exception, got {other:?}"),
        }
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/script/call_function
#[tokio::test]
async fn call_function_receives_every_local_value_kind() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let primitive = LocalValue::PrimitiveProtocolValue;
        let arguments = vec![
            string("text"),
            number(1.5),
            primitive(PrimitiveProtocolValue::BooleanValue(BooleanValue {
                value_type: "boolean".into(),
                value: true,
            })),
            primitive(PrimitiveProtocolValue::NullValue(NullValue {
                value_type: "null".into(),
            })),
            primitive(PrimitiveProtocolValue::UndefinedValue(UndefinedValue {
                value_type: "undefined".into(),
            })),
            primitive(PrimitiveProtocolValue::BigIntValue(BigIntValue {
                value_type: "bigint".into(),
                value: "12345678901234567890".into(),
            })),
            LocalValue::ArrayLocalValue(ArrayLocalValue {
                value_type: "array".into(),
                value: vec![number(1.0), string("a")],
            }),
            LocalValue::DateLocalValue(DateLocalValue {
                value_type: "date".into(),
                value: "2026-11-27T00:00:00.000Z".into(),
            }),
            LocalValue::MapLocalValue(MapLocalValue {
                value_type: "map".into(),
                value: MappingLocalValue(vec![(
                    LocalValueOrText::LocalValue(string("k")),
                    number(2.0),
                )]),
            }),
            LocalValue::ObjectLocalValue(ObjectLocalValue {
                value_type: "object".into(),
                value: MappingLocalValue(vec![(LocalValueOrText::Text("k".into()), number(3.0))]),
            }),
            LocalValue::RegExpLocalValue(RegExpLocalValue {
                value_type: "regexp".into(),
                value: RegExpValue {
                    pattern: "a+".into(),
                    flags: Some("g".into()),
                },
            }),
            LocalValue::SetLocalValue(SetLocalValue {
                value_type: "set".into(),
                value: vec![string("x")],
            }),
        ];
        let described = as_string(
            call(
                &mut session,
                &context,
                "function (...args) {
                    return JSON.stringify({
                        kinds: args.map(a => Object.prototype.toString.call(a)),
                        values: [args[0], args[1], String(args[5]), args[7].toISOString(),
                                 args[8].get('k'), args[9].k, args[10].source, [...args[11]]],
                        self: this.valueOf()
                    });
                }",
                arguments,
                Some(string("receiver")),
            )
            .await?,
        )?;
        let got: serde_json::Value = serde_json::from_str(&described)?;
        let expected = serde_json::json!({
            "kinds": ["[object String]", "[object Number]", "[object Boolean]", "[object Null]",
                      "[object Undefined]", "[object BigInt]", "[object Array]", "[object Date]",
                      "[object Map]", "[object Object]", "[object RegExp]", "[object Set]"],
            "values": ["text", 1.5, "12345678901234567890", "2026-11-27T00:00:00.000Z",
                       2, 3, "a+", ["x"]],
            "self": "receiver"
        });
        ensure!(got == expected, "got {got}");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn call_function_with_node_and_object_references() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<p id='target'>hi</p>")).await?;

        let RemoteValue::NodeRemoteValue(node) =
            evaluate(&mut session, &context, "document.getElementById('target')").await?
        else {
            bail!("expected a node");
        };
        let shared_id = node
            .shared_id
            .ok_or_else(|| anyhow!("node has no sharedId"))?;
        let by_shared_id =
            LocalValue::RemoteReference(RemoteReference::SharedReference(SharedReference {
                shared_id,
                handle: None,
                extensible: Extensible::new(),
            }));
        let id = as_string(
            call(
                &mut session,
                &context,
                "n => n.id",
                vec![by_shared_id],
                None,
            )
            .await?,
        )?;
        ensure!(id == "target", "node reference resolved to #{id}");

        let RemoteValue::ObjectRemoteValue(object) =
            evaluate(&mut session, &context, "({ answer: 42 })").await?
        else {
            bail!("expected an object");
        };
        let handle = object
            .handle
            .ok_or_else(|| anyhow!("object has no handle"))?;
        let by_handle = LocalValue::RemoteReference(RemoteReference::RemoteObjectReference(
            RemoteObjectReference {
                handle,
                shared_id: None,
                extensible: Extensible::new(),
            },
        ));
        let answer = as_string(
            call(
                &mut session,
                &context,
                "o => String(o.answer)",
                vec![by_handle],
                None,
            )
            .await?,
        )?;
        ensure!(answer == "42", "object reference resolved to {answer}");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/script/disown
#[tokio::test]
async fn disowned_handles_are_released() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let RemoteValue::ObjectRemoteValue(object) =
            evaluate(&mut session, &context, "({ answer: 42 })").await?
        else {
            bail!("expected an object");
        };
        let handle = object
            .handle
            .ok_or_else(|| anyhow!("object has no handle"))?;
        session
            .script_disown(DisownParameters::new(
                vec![handle.clone()],
                context_target(&context),
            ))
            .await?;

        let by_handle = LocalValue::RemoteReference(RemoteReference::RemoteObjectReference(
            RemoteObjectReference {
                handle,
                shared_id: None,
                extensible: Extensible::new(),
            },
        ));
        let result = session
            .script_call_function(CallFunctionParameters::new(
                "o => o.answer".into(),
                false,
                context_target(&context),
                Some(vec![by_handle]),
                None,
                None,
                None,
                None,
            ))
            .await;
        match result {
            Err(e) => ensure!(
                error_code(&e) == Some("no such handle"),
                "expected `no such handle`, got {e}"
            ),
            Ok(other) => bail!("a disowned handle still resolved: {other:?}"),
        }
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/script/add_preload_script
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/script/message
#[tokio::test]
async fn preload_script_runs_and_sends_channel_messages() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let mut messages = subscribe(&mut session, "script.message").await?;
        let channel = ChannelValue {
            value_type: "channel".into(),
            value: ChannelProperties {
                channel: "preload-channel".into(),
                serialization_options: None,
                ownership: None,
            },
        };
        let script = session
            .script_add_preload_script(AddPreloadScriptParameters::new(
                "(send) => { window.__preloaded = true; send('hello from preload'); }".into(),
                Some(vec![channel]),
                None,
                None,
                None,
            ))
            .await?
            .script;

        navigate(&mut session, &context, &inline("<p>preload</p>")).await?;
        let message: Message = messages
            .next_matching(|m: &Message| m.params.channel == "preload-channel")
            .await?;
        ensure!(
            matches!(&message.params.data,
                RemoteValue::PrimitiveProtocolValue(PrimitiveProtocolValue::StringValue(s))
                    if s.value == "hello from preload"),
            "unexpected message data: {:?}",
            message.params.data
        );
        ensure!(
            message.params.source.context.as_deref() == Some(context.as_str()),
            "message from another context"
        );
        ensure!(
            eval_string(&mut session, &context, "String(window.__preloaded)").await? == "true",
            "preload script didn't run"
        );

        session
            .script_remove_preload_script(RemovePreloadScriptParameters::new(script))
            .await?;
        navigate(&mut session, &context, &inline("<p>no preload</p>")).await?;
        ensure!(
            eval_string(&mut session, &context, "String(window.__preloaded)").await? == "undefined",
            "removed preload script still ran"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/script/get_realms
#[tokio::test]
async fn get_realms_lists_window_realms() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        let realms = session
            .script_get_realms(GetRealmsParameters::new(
                Some(context.clone()),
                Some(RealmType::Window),
            ))
            .await?
            .realms;
        ensure!(
            realms.iter().any(|realm| matches!(realm,
                RealmInfo::WindowRealmInfo(w) if w.context == context && w.sandbox.is_none())),
            "no window realm for the context: {realms:?}"
        );
        Ok(())
    })
    .await
}

#[tokio::test]
async fn sandboxes_are_isolated_from_the_page() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<p>sandbox</p>")).await?;
        let sandbox = Target::ContextTarget(ContextTarget::new(context.clone(), Some("sb".into())));
        session
            .script_evaluate(EvaluateParameters::new(
                "window.__secret = 'sandboxed'".into(),
                sandbox,
                false,
                None,
                None,
                None,
            ))
            .await?;
        ensure!(
            eval_string(&mut session, &context, "String(window.__secret)").await? == "undefined",
            "sandbox global leaked into the page"
        );
        let realms = session
            .script_get_realms(GetRealmsParameters::new(Some(context.clone()), None))
            .await?
            .realms;
        ensure!(
            realms.iter().any(|realm| matches!(realm,
                RealmInfo::WindowRealmInfo(w) if w.sandbox.as_deref() == Some("sb"))),
            "sandbox realm missing from getRealms"
        );
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/script/realm_created
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/script/realm_destroyed
#[tokio::test]
async fn realm_events_follow_navigation() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<p>first</p>")).await?;
        let old_realm = session
            .script_get_realms(GetRealmsParameters::new(Some(context.clone()), None))
            .await?
            .realms
            .into_iter()
            .find_map(|realm| match realm {
                RealmInfo::WindowRealmInfo(w) => Some(w.base.realm),
                _ => None,
            })
            .ok_or_else(|| anyhow!("no window realm"))?;
        let mut created = subscribe(&mut session, "script.realmCreated").await?;
        let mut destroyed = subscribe(&mut session, "script.realmDestroyed").await?;

        navigate(&mut session, &context, &inline("<p>second</p>")).await?;

        created
            .next_matching(|e: &RealmCreated| {
                matches!(&e.params, RealmInfo::WindowRealmInfo(w) if w.context == context)
            })
            .await?;
        destroyed
            .next_matching(|e: &RealmDestroyed| e.params.realm == old_realm)
            .await?;
        Ok(())
    })
    .await
}
