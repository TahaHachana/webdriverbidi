//! Live tests for the `script` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use anyhow::{Result, bail, ensure};
use webdriverbidi::model::script::{
    ContextTarget, EvaluateParameters, EvaluationResult, RemoteValue, RemoteValueOrText,
    ResultOwnership, Target,
};
use webdriverbidi::session::WebDriverBiDiSession;

use support::{inline, navigate, top_context, with_session};

async fn evaluate(
    session: &mut WebDriverBiDiSession,
    context: &str,
    expression: &str,
) -> Result<RemoteValue> {
    let result = session
        .script_evaluate(EvaluateParameters::new(
            expression.into(),
            Target::ContextTarget(ContextTarget::new(context.into(), None)),
            false,
            Some(ResultOwnership::Root),
            None,
            None,
        ))
        .await?;
    match result {
        EvaluationResult::EvaluationResultSuccess(success) => Ok(success.result),
        other => bail!("`{expression}` did not succeed: {other:?}"),
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
                    .ok_or_else(|| anyhow::anyhow!("node has no properties"))?;
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
                Target::ContextTarget(ContextTarget::new(context, None)),
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
