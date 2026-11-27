//! Live tests for the `session` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use anyhow::{Result, ensure};
use webdriverbidi::model::common::EmptyParams;
use webdriverbidi::model::session::{
    SubscribeParameters, UnsubscribeByAttributesRequest, UnsubscribeByIDRequest,
    UnsubscribeParameters,
};

use support::with_session;

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/session/status
#[tokio::test]
async fn status() -> Result<()> {
    with_session(|mut session| async move {
        let status = session.session_status(EmptyParams::new()).await?;
        ensure!(!status.message.is_empty(), "status message is empty");
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/session/subscribe
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/session/unsubscribe
#[tokio::test]
async fn subscribe_then_unsubscribe_by_id() -> Result<()> {
    with_session(|mut session| async move {
        let result = session
            .session_subscribe(SubscribeParameters::new(
                vec!["browsingContext.load".into()],
                None,
                None,
            ))
            .await?;
        let subscription = result
            .subscription
            .ok_or_else(|| anyhow::anyhow!("session.subscribe returned no subscription id"))?;
        session
            .session_unsubscribe(UnsubscribeParameters::UnsubscribeByIDRequest(
                UnsubscribeByIDRequest::new(vec![subscription]),
            ))
            .await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn subscribe_then_unsubscribe_by_attributes() -> Result<()> {
    with_session(|mut session| async move {
        session
            .session_subscribe(SubscribeParameters::new(
                vec!["log.entryAdded".into()],
                None,
                None,
            ))
            .await?;
        session
            .session_unsubscribe(UnsubscribeParameters::UnsubscribeByAttributesRequest(
                UnsubscribeByAttributesRequest::new(vec!["log.entryAdded".into()]),
            ))
            .await?;
        Ok(())
    })
    .await
}
