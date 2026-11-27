//! Live tests for the `input` module. Needs a WebDriver BiDi server; see `tests/support`.

mod support;

use std::time::Duration;

use anyhow::{Result, anyhow, bail, ensure};
use webdriverbidi::model::common::Extensible;
use webdriverbidi::model::input::*;
use webdriverbidi::model::script::{RemoteValue, SharedReference};
use webdriverbidi::session::WebDriverBiDiSession;

use support::{eval_string, evaluate, inline, navigate, subscribe, top_context, with_session};

async fn element(
    session: &mut WebDriverBiDiSession,
    context: &str,
    selector: &str,
) -> Result<SharedReference> {
    match evaluate(
        session,
        context,
        &format!("document.querySelector('{selector}')"),
    )
    .await?
    {
        RemoteValue::NodeRemoteValue(node) => Ok(SharedReference {
            shared_id: node
                .shared_id
                .ok_or_else(|| anyhow!("node has no sharedId"))?,
            handle: None,
            extensible: Extensible::new(),
        }),
        other => bail!("`{selector}` is not a node: {other:?}"),
    }
}

async fn perform(
    session: &mut WebDriverBiDiSession,
    context: &str,
    actions: Vec<SourceActions>,
) -> Result<()> {
    session
        .input_perform_actions(PerformActionsParameters::new(context.into(), actions))
        .await?;
    Ok(())
}

fn no_props() -> PointerCommonProperties {
    PointerCommonProperties::new(None, None, None, None, None, None, None)
}

fn click_on(target: SharedReference) -> SourceActions {
    SourceActions::PointerSourceActions(PointerSourceActions::new(
        "mouse".into(),
        Some(PointerParameters::new(Some(PointerType::Mouse))),
        vec![
            PointerSourceAction::PointerMoveAction(PointerMoveAction::new(
                0.0,
                0.0,
                None,
                Some(Origin::ElementOrigin(ElementOrigin::new(target))),
                no_props(),
            )),
            PointerSourceAction::PointerDownAction(PointerDownAction::new(0, no_props())),
            PointerSourceAction::PointerUpAction(PointerUpAction::new(0)),
        ],
    ))
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/input/perform_actions
// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/input/release_actions
#[tokio::test]
async fn key_actions_type_into_focused_input() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(&mut session, &context, &inline("<input id='field'>")).await?;
        evaluate(
            &mut session,
            &context,
            "document.getElementById('field').focus()",
        )
        .await?;

        let keys = ["R", "u", "s", "t"]
            .into_iter()
            .flat_map(|key| {
                [
                    KeySourceAction::KeyDownAction(KeyDownAction::new(key.into())),
                    KeySourceAction::KeyUpAction(KeyUpAction::new(key.into())),
                ]
            })
            .collect();
        perform(
            &mut session,
            &context,
            vec![SourceActions::KeySourceActions(KeySourceActions::new(
                "keyboard".into(),
                keys,
            ))],
        )
        .await?;
        let value = eval_string(
            &mut session,
            &context,
            "document.getElementById('field').value",
        )
        .await?;
        ensure!(value == "Rust", "input value is {value:?}");

        session
            .input_release_actions(ReleaseActionsParameters::new(context))
            .await?;
        Ok(())
    })
    .await
}

#[tokio::test]
async fn pointer_actions_click_an_element() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(
            &mut session,
            &context,
            &inline(
                "<button id='b' style='margin:100px' onclick='window.__clicks = (window.__clicks || 0) + 1'>\
                 click</button>",
            ),
        )
        .await?;
        let button = element(&mut session, &context, "#b").await?;
        perform(
            &mut session,
            &context,
            vec![
                // Exercise the none source alongside the pointer.
                SourceActions::NoneSourceActions(NoneSourceActions::new(
                    "idle".into(),
                    vec![PauseAction::new(Some(10))],
                )),
                click_on(button),
            ],
        )
        .await?;
        let clicks = eval_string(&mut session, &context, "String(window.__clicks)").await?;
        ensure!(clicks == "1", "clicks: {clicks}");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn wheel_actions_scroll_the_page() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(
            &mut session,
            &context,
            &inline("<div style='height:5000px'>tall</div>"),
        )
        .await?;
        perform(
            &mut session,
            &context,
            vec![SourceActions::WheelSourceActions(WheelSourceActions::new(
                "wheel".into(),
                vec![WheelSourceAction::WheelScrollAction(
                    WheelScrollAction::new(
                        10,
                        10,
                        0,
                        400,
                        Some(0),
                        Some(Origin::Viewport("viewport".into())),
                    ),
                )],
            ))],
        )
        .await?;
        for _ in 0..30 {
            if eval_string(&mut session, &context, "String(scrollY > 0)").await? == "true" {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        bail!("the page didn't scroll")
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/input/set_files
#[tokio::test]
async fn set_files_on_a_file_input() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(
            &mut session,
            &context,
            &inline("<input type='file' id='f' multiple>"),
        )
        .await?;
        let dir = support::temp_dir("files")?;
        let file = dir.join("upload.txt");
        std::fs::write(&file, "payload")?;

        let input = element(&mut session, &context, "#f").await?;
        session
            .input_set_files(SetFilesParameters::new(
                context.clone(),
                input,
                vec![file.to_string_lossy().into()],
            ))
            .await?;
        let names = eval_string(
            &mut session,
            &context,
            "[...document.getElementById('f').files].map(f => f.name).join(',')",
        )
        .await?;
        ensure!(names == "upload.txt", "files: {names:?}");
        let _ = std::fs::remove_dir_all(dir);
        Ok(())
    })
    .await
}

// https://github.com/web-platform-tests/wpt/tree/master/webdriver/tests/bidi/input/file_dialog_opened
#[tokio::test]
async fn file_dialog_opened_event() -> Result<()> {
    with_session(|mut session| async move {
        let context = top_context(&mut session).await?;
        navigate(
            &mut session,
            &context,
            &inline("<input type='file' id='f' style='margin:50px'>"),
        )
        .await?;
        let mut dialogs = subscribe(&mut session, "input.fileDialogOpened").await?;
        let input = element(&mut session, &context, "#f").await?;
        perform(&mut session, &context, vec![click_on(input)]).await?;

        let opened: FileDialogOpened = dialogs.next().await?;
        ensure!(
            opened.params.context == context,
            "dialog for another context"
        );
        ensure!(
            !opened.params.multiple,
            "single-file input reported as multiple"
        );
        ensure!(opened.params.element.is_some(), "event has no element");
        Ok(())
    })
    .await
}
