//! Live tests for the `session` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use anyhow::{Result, ensure};
use webdriverbidi::model::common::EmptyParams;
use webdriverbidi::model::session::{
    SubscribeParameters, UnsubscribeByAttributesRequest, UnsubscribeByIDRequest,
    UnsubscribeParameters,
};

use support::{new_session, with_session};

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

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/session/end
#[tokio::test]
async fn end_closes_the_session() -> Result<()> {
    // Not run through `with_session`, which ends the session itself.
    let mut session = new_session().await?;
    let ended = session.session_end(EmptyParams::new()).await;
    // Firefox only allows `session.end` in sessions started over BiDi.
    if support::supported("session.end", ended)?.is_none() {
        session.close().await?;
        return Ok(());
    }
    // chromedriver still accepts the HTTP `DELETE` afterwards, so its result
    // says nothing about whether `session.end` worked; ignore it.
    let _ = session.close().await;
    Ok(())
}

/// CPU time (user + system) this process has used so far, from `/proc/self/stat`.
#[cfg(target_os = "linux")]
fn process_cpu_time() -> Result<std::time::Duration> {
    let stat = std::fs::read_to_string("/proc/self/stat")?;
    // Fields after the parenthesised command name; utime and stime are the
    // 14th and 15th fields overall, i.e. the 12th and 13th after it.
    let after_name = &stat[stat.rfind(')').ok_or_else(|| anyhow::anyhow!("bad stat"))? + 2..];
    let fields: Vec<&str> = after_name.split_whitespace().collect();
    let ticks: u64 = fields[11].parse::<u64>()? + fields[12].parse::<u64>()?;
    // Clock ticks are 1/100 s on Linux (USER_HZ).
    Ok(std::time::Duration::from_millis(ticks * 10))
}

/// CPU time used while doing nothing for `idle`.
#[cfg(target_os = "linux")]
async fn cpu_used_while_idle(idle: std::time::Duration) -> Result<std::time::Duration> {
    let before = process_cpu_time()?;
    tokio::time::sleep(idle).await;
    Ok(process_cpu_time()? - before)
}

// Regression test: the receive loop used to poll the WebSocket without ever
// waiting, burning a CPU core per session, and kept spinning after the
// connection closed.
#[cfg(target_os = "linux")]
#[tokio::test]
async fn sessions_do_not_spin_while_idle_or_after_closing() -> Result<()> {
    let idle = std::time::Duration::from_secs(2);
    let limit = std::time::Duration::from_millis(300);

    let mut session = new_session().await?;
    let open = cpu_used_while_idle(idle).await?;
    session.close().await?;
    // Give the connection a moment to close before measuring.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let closed = cpu_used_while_idle(idle).await?;

    eprintln!("CPU over {idle:?}: {open:?} idle, {closed:?} after closing");
    ensure!(
        open < limit && closed < limit,
        "CPU used over {idle:?}: {open:?} with an idle session, {closed:?} after closing it (limit {limit:?})"
    );
    Ok(())
}

// A command still waiting for its response when the connection closes must fail
// promptly, instead of waiting for the 60-second response timeout.
#[tokio::test]
async fn in_flight_commands_fail_fast_when_the_connection_closes() -> Result<()> {
    let mut session = new_session().await?;
    let context = support::top_context(&mut session).await?;
    let mut waiting = session.clone();
    let in_flight = tokio::spawn(async move {
        // Never resolves, so the command waits until the connection goes away.
        support::evaluate(&mut waiting, &context, "new Promise(() => {})").await
    });
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let closed_at = std::time::Instant::now();
    session.close().await?;
    match tokio::time::timeout(std::time::Duration::from_secs(10), in_flight).await {
        Err(_) => anyhow::bail!("the in-flight command was still waiting 10s after closing"),
        Ok(result) => {
            let outcome = result?;
            ensure!(
                outcome.is_err(),
                "the in-flight command succeeded: {outcome:?}"
            );
            eprintln!(
                "in-flight command failed {:?} after closing: {}",
                closed_at.elapsed(),
                outcome.unwrap_err()
            );
        }
    }
    Ok(())
}
