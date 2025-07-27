use serde::{Deserialize, Serialize};

use crate::model::browser::UserContext;
use crate::model::browsing_context::BrowsingContext;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EmulationCommand {
    SetGeolocationOverride(SetGeolocationOverride),
    SetLocaleOverride(SetLocaleOverride),
    SetScreenOrientationOverride(SetScreenOrientationOverride),
    SetTimezoneOverride(SetTimezoneOverride),
}

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

#[derive(Debug, Serialize, Deserialize)]
pub struct SetTimezoneOverrideParameters {
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}
