use serde::{Deserialize, Serialize};

use super::id;
use super::utils;

use crate::define_command;
use crate::error::CommandError;
use crate::model::browsing_context::*;
use crate::model::result::EmptyResult;
use crate::session::WebDriverBiDiSession;

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-activate
define_command!(
    ActivateComm,
    Activate,
    ActivateParameters,
    activate,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot
define_command!(
    CaptureScreenshotCommand,
    CaptureScreenshot,
    CaptureScreenshotParameters,
    capture_screenshot,
    CaptureScreenshotResult
);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-close
define_command!(CloseCommand, Close, CloseParameters, close, EmptyResult);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-create
define_command!(
    CreateCommand,
    Create,
    CreateParameters,
    create,
    CreateResult
);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree
define_command!(
    GetTreeCommand,
    GetTree,
    GetTreeParameters,
    get_tree,
    GetTreeResult
);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-handleUserPrompt
define_command!(
    HandleUserPromptCommand,
    HandleUserPrompt,
    HandleUserPromptParameters,
    handle_user_prompt,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-locateNodes
define_command!(
    LocateNodesCommand,
    LocateNodes,
    LocateNodesParameters,
    locate_nodes,
    LocateNodesResult
);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate
define_command!(
    NavigateCommand,
    Navigate,
    NavigateParameters,
    navigate,
    NavigateResult
);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-print
define_command!(PrintCommand, Print, PrintParameters, print, PrintResult);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-reload
define_command!(
    ReloadCommand,
    Reload,
    ReloadParameters,
    reload,
    NavigateResult
);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-setViewport
define_command!(
    SetViewportCommand,
    SetViewport,
    SetViewportParameters,
    set_viewport,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-browsingContext-traverseHistory
define_command!(
    TraverseHistoryCommand,
    TraverseHistory,
    TraverseHistoryParameters,
    traverse_history,
    TraverseHistoryResult
);
