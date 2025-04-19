use serde::{Deserialize, Serialize};

use super::id;
use super::utils;

use crate::define_command;
use crate::error::CommandError;
use crate::model::result::EmptyResult;
use crate::model::web_extension::*;
use crate::session::WebDriverBiDiSession;

// https://w3c.github.io/webdriver-bidi/#command-webExtension-install
define_command!(
    InstallCommand,
    Install,
    InstallParameters,
    install,
    InstallResult
);

// https://w3c.github.io/webdriver-bidi/#command-webExtension-uninstall
define_command!(
    UninstallCommand,
    Uninstall,
    UninstallParameters,
    uninstall,
    EmptyResult
);
