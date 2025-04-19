#![allow(clippy::all)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// Type alias for extensible capabilities, which are represented as a HashMap
// with String keys and serde_json::Value values.
pub type Extensible = HashMap<String, Value>;

/// Standard WebDriver capabilities https://w3c.github.io/webdriver/#capabilities.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_insecure_certs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_load_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_window_rect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeouts: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_file_interactability: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unhandled_prompt_behavior: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    // Ensure the webSocketUrl capability is always present and set to true.
    #[serde(default = "default_web_socket_url")]
    pub web_socket_url: bool,
    // Additional extensible capabilities.
    #[serde(flatten)]
    pub extension: Extensible,
}

// Default value for the webSocketUrl capability.
fn default_web_socket_url() -> bool {
    true
}

impl CapabilityRequest {
    /// Constructs a new CapabilityRequest instance with default values.
    /// The webSocketUrl capability is set to true.
    pub fn new() -> Self {
        Self {
            browser_name: None,
            browser_version: None,
            platform_name: None,
            accept_insecure_certs: None,
            page_load_strategy: None,
            proxy: None,
            set_window_rect: None,
            timeouts: None,
            strict_file_interactability: None,
            unhandled_prompt_behavior: None,
            user_agent: None,
            web_socket_url: true,
            extension: Extensible::new(),
        }
    }
}

/// Capabilities struct to represent the standard capabilities JSON object.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CapabilitiesRequest {
    #[serde(rename = "alwaysMatch", skip_serializing_if = "Option::is_none")]
    always_match: Option<CapabilityRequest>,
    #[serde(rename = "firstMatch", skip_serializing_if = "Option::is_none")]
    first_match: Option<Vec<HashMap<String, Value>>>,
}

impl CapabilitiesRequest {
    /// Construct a new Capabilities instance ensuring the webSocketUrl capability is set to true.
    ///
    /// # Arguments
    ///
    /// * `always_match` - A CapabilityRequest instance to be used as the alwaysMatch capability.
    ///
    /// # Returns
    ///
    /// A Capabilities instance with the alwaysMatch capability set.
    pub fn new(mut always_match: CapabilityRequest) -> Self {
        always_match.web_socket_url = true;
        Self {
            always_match: Some(always_match),
            first_match: None,
        }
    }

    /// Add a firstMatch capability to the Capabilities instance.
    ///
    /// # Arguments
    ///
    /// * `first_match` - A HashMap<String, Value> representing the firstMatch capability.
    ///
    /// # Returns
    ///
    /// A Capabilities instance with the firstMatch capability added.
    pub fn add_first_match(&mut self, first_match: HashMap<String, Value>) {
        if let Some(ref mut first_match_vec) = self.first_match {
            first_match_vec.push(first_match);
        } else {
            self.first_match = Some(vec![first_match]);
        }
    }

    /// Add a non standard alwaysMatch capability if the key is not `webSocketUrl`.
    ///
    /// # Arguments
    ///
    /// * `key` - A String representing the capability key.
    /// * `value` - A serde_json::Value representing the capability value.
    ///
    /// # Returns
    ///
    /// A Capabilities instance with the alwaysMatch capability added.
    pub fn add_extension(&mut self, key: String, value: Value) {
        if key != "webSocketUrl" {
            if let Some(ref mut always_match) = self.always_match {
                always_match.extension.insert(key, value);
            }
        }
    }

    /// Build the Capabilities instance into a serde_json::Value ready for serialization.
    pub fn build(&self) -> Value {
        let json = json!({
            "capabilities": self
        });
        json
    }

    /// Construct a new CapabilitiesRequest instance with default values.
    pub fn default() -> Self {
        let always_match = CapabilityRequest::new();
        CapabilitiesRequest::new(always_match)
    }
}
