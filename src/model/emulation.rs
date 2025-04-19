use serde::{Deserialize, Serialize};

use crate::model::browser::UserContext;
use crate::model::browsing_context::BrowsingContext;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EmulationCommand {
    SetGeolocationOverride(SetGeolocationOverride),
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
pub struct SetGeolocationOverrideParameters {
    pub coordinates: Option<GeolocationCoordinates>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    user_contexts: Option<Vec<UserContext>>,
}

impl SetGeolocationOverrideParameters {
    pub fn new(
        coordinates: Option<GeolocationCoordinates>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            coordinates,
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
