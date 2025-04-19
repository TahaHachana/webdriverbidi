use serde::{Deserialize, Serialize};

use super::id;
use super::utils;

use crate::define_command;
use crate::error::CommandError;
use crate::model::input::*;
use crate::model::result::EmptyResult;
use crate::session::WebDriverBiDiSession;

// https://w3c.github.io/webdriver-bidi/#command-input-performActions
define_command!(
    PerformActionsCommand,
    PerformActions,
    PerformActionsParameters,
    perform_actions,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-input-releaseActions
define_command!(
    ReleaseActionsCommand,
    ReleaseActions,
    ReleaseActionsParameters,
    release_actions,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-input-setFiles
define_command!(
    SetFilesCommand,
    SetFiles,
    SetFilesParameters,
    set_files,
    EmptyResult
);
