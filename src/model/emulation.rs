use serde::{Deserialize, Serialize};

use crate::model::browser::UserContext;
use crate::model::browsing_context::BrowsingContext;
use crate::model::common::JsUint;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EmulationCommand {
    SetForcedColorsModeThemeOverride(SetForcedColorsModeThemeOverride),
    SetGeolocationOverride(SetGeolocationOverride),
    SetLocaleOverride(SetLocaleOverride),
    SetMediaFeaturesOverride(SetMediaFeaturesOverride),
    SetNetworkConditions(SetNetworkConditions),
    SetScreenOrientationOverride(SetScreenOrientationOverride),
    SetScreenSettingsOverride(SetScreenSettingsOverride),
    SetScriptingEnabled(SetScriptingEnabled),
    SetScrollbarTypeOverride(SetScrollbarTypeOverride),
    SetTextLayoutModeOverride(SetTextLayoutModeOverride),
    SetTimezoneOverride(SetTimezoneOverride),
    SetTouchOverride(SetTouchOverride),
    SetUserAgentOverride(SetUserAgentOverride),
    SetViewportMetaOverride(SetViewportMetaOverride),
}

/// All emulation commands return an empty result.
pub type EmulationResult = crate::model::result::EmptyResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetGeolocationOverride {
    pub method: String,
    pub params: SetGeolocationOverrideParameters,
}

impl SetGeolocationOverride {
    pub fn new(params: SetGeolocationOverrideParameters) -> Self {
        Self {
            method: "emulation.setGeolocationOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SetGeolocationOverrideParameters {
    WithCoordinates {
        coordinates: Option<GeolocationCoordinates>,
        #[serde(skip_serializing_if = "Option::is_none")]
        contexts: Option<Vec<BrowsingContext>>,
        #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
        user_contexts: Option<Vec<UserContext>>,
    },
    WithError {
        error: GeolocationPositionError,
        #[serde(skip_serializing_if = "Option::is_none")]
        contexts: Option<Vec<BrowsingContext>>,
        #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
        user_contexts: Option<Vec<UserContext>>,
    },
}

impl SetGeolocationOverrideParameters {
    pub fn with_coordinates(
        coordinates: Option<GeolocationCoordinates>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self::WithCoordinates {
            coordinates,
            contexts,
            user_contexts,
        }
    }

    pub fn with_error(
        error: GeolocationPositionError,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self::WithError {
            error,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeolocationCoordinates {
    pub latitude: f64,
    pub longitude: f64,

    #[serde(default = "GeolocationCoordinates::default_accuracy")]
    pub accuracy: f64,

    #[serde(default)]
    pub altitude: Option<f64>,

    #[serde(default, rename = "altitudeAccuracy")]
    pub altitude_accuracy: Option<f64>,

    #[serde(default)]
    pub heading: Option<f64>,

    #[serde(default)]
    pub speed: Option<f64>,
}

impl GeolocationCoordinates {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            accuracy: 1.0,
            altitude: None,
            altitude_accuracy: None,
            heading: None,
            speed: None,
        }
    }

    fn default_accuracy() -> f64 {
        1.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeolocationPositionError {
    #[serde(rename = "type")]
    pub error_type: String,
}

impl GeolocationPositionError {
    pub fn position_unavailable() -> Self {
        Self {
            error_type: "positionUnavailable".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetLocaleOverride {
    pub method: String,
    pub params: SetLocaleOverrideParameters,
}

impl SetLocaleOverride {
    pub fn new(params: SetLocaleOverrideParameters) -> Self {
        Self {
            method: "emulation.setLocaleOverride".to_string(),
            params,
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SetLocaleOverrideParameters {
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetScreenOrientationOverride {
    pub method: String,
    pub params: SetScreenOrientationOverrideParameters,
}

impl SetScreenOrientationOverride {
    pub fn new(params: SetScreenOrientationOverrideParameters) -> Self {
        Self {
            method: "emulation.setScreenOrientationOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", untagged)]
pub enum ScreenOrientationNatural {
    Portrait,
    Landscape,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum ScreenOrientationType {
    PortraitPrimary,
    PortraitSecondary,
    LandscapePrimary,
    LandscapeSecondary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenOrientation {
    pub natural: ScreenOrientationNatural,
    #[serde(rename = "type")]
    pub orientation_type: ScreenOrientationType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetScreenOrientationOverrideParameters {
    #[serde(rename = "screenOrientation")]
    pub screen_orientation: Option<ScreenOrientation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTimezoneOverride {
    pub method: String,
    pub params: SetTimezoneOverrideParameters,
}

impl SetTimezoneOverride {
    pub fn new(params: SetTimezoneOverrideParameters) -> Self {
        Self {
            method: "emulation.setTimezoneOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTimezoneOverrideParameters {
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetForcedColorsModeThemeOverride {
    pub method: String,
    pub params: SetForcedColorsModeThemeOverrideParameters,
}

impl SetForcedColorsModeThemeOverride {
    pub fn new(params: SetForcedColorsModeThemeOverrideParameters) -> Self {
        Self {
            method: "emulation.setForcedColorsModeThemeOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetForcedColorsModeThemeOverrideParameters {
    /// `None` clears the override.
    pub theme: Option<ForcedColorsModeTheme>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetForcedColorsModeThemeOverrideParameters {
    pub fn new(
        theme: Option<ForcedColorsModeTheme>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            theme,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetMediaFeaturesOverride {
    pub method: String,
    pub params: SetMediaFeaturesOverrideParameters,
}

impl SetMediaFeaturesOverride {
    pub fn new(params: SetMediaFeaturesOverrideParameters) -> Self {
        Self {
            method: "emulation.setMediaFeaturesOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetMediaFeaturesOverrideParameters {
    /// `None` clears the override.
    pub features: Option<MediaFeatures>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetMediaFeaturesOverrideParameters {
    pub fn new(
        features: Option<MediaFeatures>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            features,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetNetworkConditions {
    pub method: String,
    pub params: SetNetworkConditionsParameters,
}

impl SetNetworkConditions {
    pub fn new(params: SetNetworkConditionsParameters) -> Self {
        Self {
            method: "emulation.setNetworkConditions".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetNetworkConditionsParameters {
    /// `None` clears the override.
    #[serde(rename = "networkConditions")]
    pub network_conditions: Option<NetworkConditions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetNetworkConditionsParameters {
    pub fn new(
        network_conditions: Option<NetworkConditions>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            network_conditions,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetScreenSettingsOverride {
    pub method: String,
    pub params: SetScreenSettingsOverrideParameters,
}

impl SetScreenSettingsOverride {
    pub fn new(params: SetScreenSettingsOverrideParameters) -> Self {
        Self {
            method: "emulation.setScreenSettingsOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetScreenSettingsOverrideParameters {
    /// `None` clears the override.
    #[serde(rename = "screenArea")]
    pub screen_area: Option<ScreenArea>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetScreenSettingsOverrideParameters {
    pub fn new(
        screen_area: Option<ScreenArea>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            screen_area,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetScriptingEnabled {
    pub method: String,
    pub params: SetScriptingEnabledParameters,
}

impl SetScriptingEnabled {
    pub fn new(params: SetScriptingEnabledParameters) -> Self {
        Self {
            method: "emulation.setScriptingEnabled".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetScriptingEnabledParameters {
    /// `Some(ScriptingEnabled::Disabled)` disables scripting; `None` restores the default.
    pub enabled: Option<ScriptingEnabled>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetScriptingEnabledParameters {
    pub fn new(
        enabled: Option<ScriptingEnabled>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            enabled,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetScrollbarTypeOverride {
    pub method: String,
    pub params: SetScrollbarTypeOverrideParameters,
}

impl SetScrollbarTypeOverride {
    pub fn new(params: SetScrollbarTypeOverrideParameters) -> Self {
        Self {
            method: "emulation.setScrollbarTypeOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetScrollbarTypeOverrideParameters {
    /// `None` clears the override.
    #[serde(rename = "scrollbarType")]
    pub scrollbar_type: Option<ScrollbarType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetScrollbarTypeOverrideParameters {
    pub fn new(
        scrollbar_type: Option<ScrollbarType>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            scrollbar_type,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTextLayoutModeOverride {
    pub method: String,
    pub params: SetTextLayoutModeOverrideParameters,
}

impl SetTextLayoutModeOverride {
    pub fn new(params: SetTextLayoutModeOverrideParameters) -> Self {
        Self {
            method: "emulation.setTextLayoutModeOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTextLayoutModeOverrideParameters {
    /// `None` clears the override.
    #[serde(rename = "textLayoutMode")]
    pub text_layout_mode: Option<TextLayoutMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetTextLayoutModeOverrideParameters {
    pub fn new(
        text_layout_mode: Option<TextLayoutMode>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            text_layout_mode,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTouchOverride {
    pub method: String,
    pub params: SetTouchOverrideParameters,
}

impl SetTouchOverride {
    pub fn new(params: SetTouchOverrideParameters) -> Self {
        Self {
            method: "emulation.setTouchOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTouchOverrideParameters {
    /// Maximum touch points (>= 1); `None` clears the override.
    #[serde(rename = "maxTouchPoints")]
    pub max_touch_points: Option<JsUint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetTouchOverrideParameters {
    pub fn new(
        max_touch_points: Option<JsUint>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            max_touch_points,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetUserAgentOverride {
    pub method: String,
    pub params: SetUserAgentOverrideParameters,
}

impl SetUserAgentOverride {
    pub fn new(params: SetUserAgentOverrideParameters) -> Self {
        Self {
            method: "emulation.setUserAgentOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetUserAgentOverrideParameters {
    /// `None` clears the override.
    #[serde(rename = "userAgent")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetUserAgentOverrideParameters {
    pub fn new(
        user_agent: Option<String>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            user_agent,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetViewportMetaOverride {
    pub method: String,
    pub params: SetViewportMetaOverrideParameters,
}

impl SetViewportMetaOverride {
    pub fn new(params: SetViewportMetaOverrideParameters) -> Self {
        Self {
            method: "emulation.setViewportMetaOverride".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetViewportMetaOverrideParameters {
    /// `Some(ViewportMeta::Enabled)` enables `<meta name=viewport>` handling; `None` clears the override.
    #[serde(rename = "viewportMeta")]
    pub viewport_meta: Option<ViewportMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetViewportMetaOverrideParameters {
    pub fn new(
        viewport_meta: Option<ViewportMeta>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            viewport_meta,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForcedColorsModeTheme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NetworkConditions {
    Offline,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenArea {
    pub width: JsUint,
    pub height: JsUint,
}

impl ScreenArea {
    pub fn new(width: JsUint, height: JsUint) -> Self {
        Self { width, height }
    }
}

/// The only non-null value of `emulation.SetScriptingEnabledParameters.enabled` is `false`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScriptingEnabled {
    Disabled,
}

impl Serialize for ScriptingEnabled {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(false)
    }
}

impl<'de> Deserialize<'de> for ScriptingEnabled {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match bool::deserialize(deserializer)? {
            false => Ok(ScriptingEnabled::Disabled),
            true => Err(serde::de::Error::custom("expected `false`")),
        }
    }
}

/// The only non-null value of `emulation.SetViewportMetaOverrideParameters.viewportMeta` is `true`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewportMeta {
    Enabled,
}

impl Serialize for ViewportMeta {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(true)
    }
}

impl<'de> Deserialize<'de> for ViewportMeta {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match bool::deserialize(deserializer)? {
            true => Ok(ViewportMeta::Enabled),
            false => Err(serde::de::Error::custom("expected `true`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScrollbarType {
    #[serde(rename = "classic")]
    Classic,
    #[serde(rename = "overlay")]
    Overlay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextLayoutMode {
    #[serde(rename = "mobile")]
    Mobile,
}

/// `emulation.MediaFeatures`.
///
/// Each field is `Option<Option<T>>`: the outer `None` omits the feature, `Some(None)`
/// sends `null` (reset that feature), `Some(Some(v))` overrides it.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MediaFeatures {
    #[serde(rename = "any-hover", default, skip_serializing_if = "Option::is_none")]
    pub any_hover: Option<Option<Hover>>,
    #[serde(
        rename = "any-pointer",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub any_pointer: Option<Option<Pointer>>,
    #[serde(rename = "color", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Option<JsUint>>,
    #[serde(
        rename = "color-gamut",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_gamut: Option<Option<ColorGamut>>,
    #[serde(
        rename = "color-index",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub color_index: Option<Option<JsUint>>,
    #[serde(
        rename = "display-mode",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_mode: Option<Option<DisplayMode>>,
    #[serde(
        rename = "dynamic-range",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub dynamic_range: Option<Option<DynamicRange>>,
    #[serde(
        rename = "environment-blending",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub environment_blending: Option<Option<EnvironmentBlending>>,
    #[serde(
        rename = "forced-colors",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub forced_colors: Option<Option<ForcedColors>>,
    #[serde(rename = "grid", default, skip_serializing_if = "Option::is_none")]
    pub grid: Option<Option<u8>>,
    #[serde(
        rename = "horizontal-viewport-segments",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub horizontal_viewport_segments: Option<Option<JsUint>>,
    #[serde(rename = "hover", default, skip_serializing_if = "Option::is_none")]
    pub hover: Option<Option<Hover>>,
    #[serde(
        rename = "inverted-colors",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub inverted_colors: Option<Option<InvertedColors>>,
    #[serde(
        rename = "monochrome",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub monochrome: Option<Option<JsUint>>,
    #[serde(
        rename = "nav-controls",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub nav_controls: Option<Option<NavControls>>,
    #[serde(
        rename = "overflow-block",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub overflow_block: Option<Option<OverflowBlock>>,
    #[serde(
        rename = "overflow-inline",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub overflow_inline: Option<Option<OverflowInline>>,
    #[serde(rename = "pointer", default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<Option<Pointer>>,
    #[serde(
        rename = "prefers-color-scheme",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prefers_color_scheme: Option<Option<PrefersColorScheme>>,
    #[serde(
        rename = "prefers-contrast",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prefers_contrast: Option<Option<PrefersContrast>>,
    #[serde(
        rename = "prefers-reduced-data",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prefers_reduced_data: Option<Option<PrefersReduced>>,
    #[serde(
        rename = "prefers-reduced-motion",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prefers_reduced_motion: Option<Option<PrefersReduced>>,
    #[serde(
        rename = "prefers-reduced-transparency",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub prefers_reduced_transparency: Option<Option<PrefersReduced>>,
    #[serde(rename = "scan", default, skip_serializing_if = "Option::is_none")]
    pub scan: Option<Option<Scan>>,
    #[serde(rename = "scripting", default, skip_serializing_if = "Option::is_none")]
    pub scripting: Option<Option<Scripting>>,
    #[serde(rename = "update", default, skip_serializing_if = "Option::is_none")]
    pub update: Option<Option<Update>>,
    #[serde(
        rename = "vertical-viewport-segments",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub vertical_viewport_segments: Option<Option<JsUint>>,
    #[serde(
        rename = "video-color-gamut",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub video_color_gamut: Option<Option<ColorGamut>>,
    #[serde(
        rename = "video-dynamic-range",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub video_dynamic_range: Option<Option<DynamicRange>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Hover {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "hover")]
    Hover,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pointer {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "coarse")]
    Coarse,
    #[serde(rename = "fine")]
    Fine,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorGamut {
    #[serde(rename = "srgb")]
    Srgb,
    #[serde(rename = "p3")]
    P3,
    #[serde(rename = "rec2020")]
    Rec2020,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisplayMode {
    #[serde(rename = "fullscreen")]
    Fullscreen,
    #[serde(rename = "standalone")]
    Standalone,
    #[serde(rename = "minimal-ui")]
    MinimalUi,
    #[serde(rename = "browser")]
    Browser,
    #[serde(rename = "picture-in-picture")]
    PictureInPicture,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DynamicRange {
    #[serde(rename = "standard")]
    Standard,
    #[serde(rename = "high")]
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnvironmentBlending {
    #[serde(rename = "opaque")]
    Opaque,
    #[serde(rename = "additive")]
    Additive,
    #[serde(rename = "subtractive")]
    Subtractive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForcedColors {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "active")]
    Active,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InvertedColors {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "inverted")]
    Inverted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NavControls {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "back")]
    Back,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OverflowBlock {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "scroll")]
    Scroll,
    #[serde(rename = "optional-paged")]
    OptionalPaged,
    #[serde(rename = "paged")]
    Paged,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OverflowInline {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "scroll")]
    Scroll,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrefersColorScheme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrefersContrast {
    #[serde(rename = "no-preference")]
    NoPreference,
    #[serde(rename = "more")]
    More,
    #[serde(rename = "less")]
    Less,
    #[serde(rename = "custom")]
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrefersReduced {
    #[serde(rename = "no-preference")]
    NoPreference,
    #[serde(rename = "reduce")]
    Reduce,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Scan {
    #[serde(rename = "interlace")]
    Interlace,
    #[serde(rename = "progressive")]
    Progressive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Scripting {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "initial-only")]
    InitialOnly,
    #[serde(rename = "enabled")]
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Update {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "slow")]
    Slow,
    #[serde(rename = "fast")]
    Fast,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn media_features_serialization() {
        let features = MediaFeatures {
            prefers_color_scheme: Some(Some(PrefersColorScheme::Dark)),
            prefers_reduced_motion: Some(None),
            any_pointer: Some(Some(Pointer::Coarse)),
            grid: Some(Some(1)),
            ..Default::default()
        };
        let params = SetMediaFeaturesOverrideParameters::new(Some(features), None, None);
        assert_eq!(
            serde_json::to_value(&params).unwrap(),
            json!({"features": {
                "any-pointer": "coarse",
                "grid": 1,
                "prefers-color-scheme": "dark",
                "prefers-reduced-motion": null
            }})
        );
        let reset = SetMediaFeaturesOverrideParameters::new(None, None, Some(vec!["u".into()]));
        assert_eq!(
            serde_json::to_value(&reset).unwrap(),
            json!({"features": null, "userContexts": ["u"]})
        );
    }

    #[test]
    fn literal_value_overrides() {
        let p = SetScriptingEnabledParameters::new(Some(ScriptingEnabled::Disabled), None, None);
        assert_eq!(serde_json::to_value(&p).unwrap(), json!({"enabled": false}));
        let p = SetViewportMetaOverrideParameters::new(Some(ViewportMeta::Enabled), None, None);
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            json!({"viewportMeta": true})
        );
        let p = SetNetworkConditionsParameters::new(Some(NetworkConditions::Offline), None, None);
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            json!({"networkConditions": {"type": "offline"}})
        );
        let p = SetScrollbarTypeOverrideParameters::new(Some(ScrollbarType::Overlay), None, None);
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            json!({"scrollbarType": "overlay"})
        );
    }
}
