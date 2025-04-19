use serde::{Deserialize, Serialize};

use crate::model::browser::UserContext;
use crate::model::browsing_context::BrowsingContext;
use crate::model::common::{EmptyParams, Extensible};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SessionCommand {
    End(End),
    New(New),
    Status(Status),
    Subscribe(Subscribe),
    Unsubscribe(Unsubscribe),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SessionResult {
    NewResult(NewResult),
    StatusResult(StatusResult),
    SubscribeResult(SubscribeResult),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CapabilitiesRequest {
    #[serde(rename = "alwaysMatch", skip_serializing_if = "Option::is_none")]
    pub always_match: Option<CapabilityRequest>,
    #[serde(rename = "firstMatch", skip_serializing_if = "Option::is_none")]
    pub first_match: Option<Vec<CapabilityRequest>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CapabilityRequest {
    #[serde(
        rename = "acceptInsecureCerts",
        skip_serializing_if = "Option::is_none"
    )]
    pub accept_insecure_certs: Option<bool>,
    #[serde(rename = "browserName", skip_serializing_if = "Option::is_none")]
    pub browser_name: Option<String>,
    #[serde(rename = "browserVersion", skip_serializing_if = "Option::is_none")]
    pub browser_version: Option<String>,
    #[serde(rename = "platformName", skip_serializing_if = "Option::is_none")]
    pub platform_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyConfiguration>,
    #[serde(
        rename = "unhandledPromptBehavior",
        skip_serializing_if = "Option::is_none"
    )]
    pub unhandled_prompt_behavior: Option<UserPromptHandler>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ProxyConfiguration {
    AutodetectProxyConfiguration(AutodetectProxyConfiguration),
    DirectProxyConfiguration(DirectProxyConfiguration),
    ManualProxyConfiguration(ManualProxyConfiguration),
    PacProxyConfiguration(PacProxyConfiguration),
    SystemProxyConfiguration(SystemProxyConfiguration),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AutodetectProxyConfiguration {
    #[serde(rename = "proxyType")]
    pub proxy_type: String,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectProxyConfiguration {
    #[serde(rename = "proxyType")]
    pub proxy_type: String,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ManualProxyConfiguration {
    #[serde(rename = "proxyType")]
    pub proxy_type: String,
    #[serde(rename = "ftpProxy", skip_serializing_if = "Option::is_none")]
    pub ftp_proxy: Option<String>,
    #[serde(rename = "httpProxy", skip_serializing_if = "Option::is_none")]
    pub http_proxy: Option<String>,
    #[serde(rename = "sslProxy", skip_serializing_if = "Option::is_none")]
    pub ssl_proxy: Option<String>,
    #[serde(rename = "socksProxy", skip_serializing_if = "Option::is_none")]
    pub socks_proxy: Option<SocksProxyConfiguration>,
    #[serde(rename = "noProxy", skip_serializing_if = "Option::is_none")]
    pub no_proxy: Option<Vec<String>>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SocksProxyConfiguration {
    #[serde(rename = "socksProxy")]
    pub socks_proxy: String,
    #[serde(rename = "socksVersion")]
    pub socks_version: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PacProxyConfiguration {
    #[serde(rename = "proxyType")]
    pub proxy_type: String,
    #[serde(rename = "proxyAutoconfigUrl")]
    pub proxy_autoconfig_url: String,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemProxyConfiguration {
    #[serde(rename = "proxyType")]
    pub proxy_type: String,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserPromptHandler {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<UserPromptHandlerType>,
    #[serde(rename = "beforeUnload", skip_serializing_if = "Option::is_none")]
    pub before_unload: Option<UserPromptHandlerType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confirm: Option<UserPromptHandlerType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<UserPromptHandlerType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<UserPromptHandlerType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<UserPromptHandlerType>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum UserPromptHandlerType {
    Accept,
    Dismiss,
    Ignore,
}

type Subscription = String;
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    pub events: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "userContexts")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SubscriptionRequest {
    pub fn new(
        events: Vec<String>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            events,
            contexts,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnsubscribeByIDRequest {
    pub subscriptions: Vec<Subscription>,
}

impl UnsubscribeByIDRequest {
    pub fn new(subscriptions: Vec<Subscription>) -> Self {
        Self { subscriptions }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UnsubscribeByAttributesRequest {
    pub events: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
}

impl UnsubscribeByAttributesRequest {
    pub fn new(events: Vec<String>, contexts: Option<Vec<BrowsingContext>>) -> Self {
        Self { events, contexts }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub method: String,
    pub params: EmptyParams,
}

impl Status {
    pub fn new(params: EmptyParams) -> Self {
        Self {
            method: "session.status".to_string(),
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResult {
    pub ready: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct New {
    pub method: String,
    pub params: NewParameters,
}

impl New {
    pub fn new(params: NewParameters) -> Self {
        Self {
            method: "session.new".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewParameters {
    pub capabilities: CapabilitiesRequest,
}

impl NewParameters {
    pub fn new(capabilities: CapabilitiesRequest) -> Self {
        Self { capabilities }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewResult {
    pub session_id: String,
    pub capabilities: Capabilities,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Capabilities {
    #[serde(rename = "acceptInsecureCerts")]
    pub accept_insecure_certs: bool,
    #[serde(rename = "browserName")]
    pub browser_name: String,
    #[serde(rename = "browserVersion")]
    pub browser_version: String,
    #[serde(rename = "platformName")]
    pub platform_name: String,
    #[serde(rename = "setWindowRect")]
    pub set_window_rect: bool,
    #[serde(rename = "userAgent")]
    pub user_agent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyConfiguration>,
    #[serde(
        rename = "unhandledPromptBehavior",
        skip_serializing_if = "Option::is_none"
    )]
    pub unhandled_prompt_behavior: Option<UserPromptHandler>,
    #[serde(rename = "webSocketUrl", skip_serializing_if = "Option::is_none")]
    pub web_socket_url: Option<String>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct End {
    pub method: String,
    pub params: EmptyParams,
}

impl End {
    pub fn new(params: EmptyParams) -> Self {
        Self {
            method: "session.end".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscribe {
    pub method: String,
    pub params: SubscriptionRequest,
}

impl Subscribe {
    pub fn new(params: SubscriptionRequest) -> Self {
        Self {
            method: "session.subscribe".to_string(),
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription: Option<Subscription>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Unsubscribe {
    pub method: String,
    pub params: UnsubscribeParameters,
}

impl Unsubscribe {
    pub fn new(params: UnsubscribeParameters) -> Self {
        Self {
            method: "session.unsubscribe".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnsubscribeParameters {
    UnsubscribeByAttributesRequest(UnsubscribeByAttributesRequest),
    UnsubscribeByIDRequest(UnsubscribeByIDRequest),
}
