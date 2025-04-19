use serde::{Deserialize, Serialize};

use super::id;
use super::utils;

use crate::define_command;
use crate::error::CommandError;
use crate::model::emulation::*;
use crate::model::result::EmptyResult;
use crate::session::WebDriverBiDiSession;

// https://w3c.github.io/webdriver-bidi/#command-emulation-setGeolocationOverride
define_command!(
    SetGeolocationOverrideCommand,
    SetGeolocationOverride,
    SetGeolocationOverrideParameters,
    set_geolocation_override,
    EmptyResult
);
