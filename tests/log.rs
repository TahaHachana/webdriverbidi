//! Live tests for the `log` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use anyhow::{Result, bail, ensure};
use webdriverbidi::model::log::{Entry, EntryAdded, Level};

use support::{evaluate, inline, navigate, subscribe, top_context, with_session};

fn text(entry: &EntryAdded) -> &str {
    let base = match &entry.params {
        Entry::ConsoleLogEntry(e) => &e.base,
        Entry::JavascriptLogEntry(e) => &e.base,
        Entry::GenericLogEntry(e) => &e.base,
    };
    base.text.as_deref().unwrap_or_default()
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/log/entry_added
#[tokio::test]
async fn console_entries() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<p>console</p>")).await?;
        let mut entries = subscribe(&mut session, "log.entryAdded").await?;

        evaluate(&mut session, &context, "console.log('hello', 42)").await?;
        let entry: EntryAdded = entries
            .next_matching(|e: &EntryAdded| text(e).contains("hello"))
            .await?;
        match &entry.params {
            Entry::ConsoleLogEntry(console) => {
                ensure!(console.method == "log", "method is {}", console.method);
                ensure!(console.args.len() == 2, "{} args", console.args.len());
                ensure!(
                    matches!(console.base.level, Level::Info),
                    "level {:?}",
                    console.base.level
                );
                ensure!(
                    console.base.source.context.as_deref() == Some(context.as_str()),
                    "entry from another context"
                );
            }
            other => bail!("expected a console entry, got {other:?}"),
        }

        evaluate(&mut session, &context, "console.error('broken')").await?;
        let entry: EntryAdded = entries
            .next_matching(|e: &EntryAdded| text(e).contains("broken"))
            .await?;
        match &entry.params {
            Entry::ConsoleLogEntry(console) => {
                ensure!(
                    matches!(console.base.level, Level::Error),
                    "level {:?}",
                    console.base.level
                );
            }
            other => bail!("expected a console entry, got {other:?}"),
        }
        Ok(())
    })
    .await
}

#[tokio::test]
async fn uncaught_exceptions_are_javascript_entries() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<p>exceptions</p>")).await?;
        let mut entries = subscribe(&mut session, "log.entryAdded").await?;

        evaluate(
            &mut session,
            &context,
            "setTimeout(() => { throw new Error('uncaught boom') }, 0)",
        )
        .await?;
        let entry: EntryAdded = entries
            .next_matching(|e: &EntryAdded| text(e).contains("uncaught boom"))
            .await?;
        match &entry.params {
            Entry::JavascriptLogEntry(js) => {
                ensure!(
                    matches!(js.base.level, Level::Error),
                    "level {:?}",
                    js.base.level
                );
                ensure!(js.base.stack_trace.is_some(), "no stack trace");
            }
            other => bail!("expected a javascript entry, got {other:?}"),
        }
        Ok(())
    })
    .await
}
