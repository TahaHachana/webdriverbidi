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

// https://w3c.github.io/webdriver-bidi/#command-emulation-setForcedColorsModeThemeOverride
define_command!(
    SetForcedColorsModeThemeOverrideCommand,
    SetForcedColorsModeThemeOverride,
    SetForcedColorsModeThemeOverrideParameters,
    set_forced_colors_mode_theme_override,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setMediaFeaturesOverride
define_command!(
    SetMediaFeaturesOverrideCommand,
    SetMediaFeaturesOverride,
    SetMediaFeaturesOverrideParameters,
    set_media_features_override,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setNetworkConditions
define_command!(
    SetNetworkConditionsCommand,
    SetNetworkConditions,
    SetNetworkConditionsParameters,
    set_network_conditions,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setScreenSettingsOverride
define_command!(
    SetScreenSettingsOverrideCommand,
    SetScreenSettingsOverride,
    SetScreenSettingsOverrideParameters,
    set_screen_settings_override,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setScriptingEnabled
define_command!(
    SetScriptingEnabledCommand,
    SetScriptingEnabled,
    SetScriptingEnabledParameters,
    set_scripting_enabled,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setScrollbarTypeOverride
define_command!(
    SetScrollbarTypeOverrideCommand,
    SetScrollbarTypeOverride,
    SetScrollbarTypeOverrideParameters,
    set_scrollbar_type_override,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setTextLayoutModeOverride
define_command!(
    SetTextLayoutModeOverrideCommand,
    SetTextLayoutModeOverride,
    SetTextLayoutModeOverrideParameters,
    set_text_layout_mode_override,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setTouchOverride
define_command!(
    SetTouchOverrideCommand,
    SetTouchOverride,
    SetTouchOverrideParameters,
    set_touch_override,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setUserAgentOverride
define_command!(
    SetUserAgentOverrideCommand,
    SetUserAgentOverride,
    SetUserAgentOverrideParameters,
    set_user_agent_override,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-emulation-setViewportMetaOverride
define_command!(
    SetViewportMetaOverrideCommand,
    SetViewportMetaOverride,
    SetViewportMetaOverrideParameters,
    set_viewport_meta_override,
    EmptyResult
);
