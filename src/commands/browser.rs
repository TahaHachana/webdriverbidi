use serde::{Deserialize, Serialize};

use super::id;
use super::utils;

use crate::define_command;
use crate::error::CommandError;
use crate::model::browser::ClientWindowInfo;
use crate::model::browser::*;
use crate::model::common::EmptyParams;
use crate::model::result::EmptyResult;
use crate::session::WebDriverBiDiSession;

// https://w3c.github.io/webdriver-bidi/#command-browser-close
define_command!(CloseCommand, Close, EmptyParams, close, EmptyResult);

// https://w3c.github.io/webdriver-bidi/#command-browser-createUserContext
define_command!(
    CreateUserContextCommand,
    CreateUserContext,
    EmptyParams,
    create_user_context,
    CreateUserContextResult
);

// https://w3c.github.io/webdriver-bidi/#command-browser-getClientWindows
define_command!(
    GetClientWindowsCommand,
    GetClientWindows,
    EmptyParams,
    get_client_windows,
    GetClientWindowsResult
);

// https://w3c.github.io/webdriver-bidi/#command-browser-getUserContexts
define_command!(
    GetUserContextsCommand,
    GetUserContexts,
    EmptyParams,
    get_user_contexts,
    GetUserContextsResult
);

// https://w3c.github.io/webdriver-bidi/#command-browser-removeUserContext
define_command!(
    RemoveUserContextCommand,
    RemoveUserContext,
    RemoveUserContextParameters,
    remove_user_context,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-browser-setClientWindowState
define_command!(
    SetClientWindowStateCommand,
    SetClientWindowState,
    SetClientWindowStateParameters,
    set_client_window_state,
    ClientWindowInfo
);
