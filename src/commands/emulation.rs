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

// https://w3c.github.io/webdriver-bidi/#command-emulation-setLocaleOverride
define_command!(
    SetLocaleOverrideCommand,
    SetLocaleOverride,
    SetLocaleOverrideParameters,
    set_locale_override,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setScreenOrientationOverride
define_command!(
    SetScreenOrientationOverrideCommand,
    SetScreenOrientationOverride,
    SetScreenOrientationOverrideParameters,
    set_screen_orientation_override,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setTimezoneOverride
define_command!(
    SetTimezoneOverrideCommand,
    SetTimezoneOverride,
    SetTimezoneOverrideParameters,
    set_timezone_override,
    EmptyResult
);
