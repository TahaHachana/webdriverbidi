use serde::{Deserialize, Serialize};

use crate::model::common::{EmptyParams, JsInt, JsUint};
use crate::model::session::{ProxyConfiguration, UserPromptHandler};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum BrowserCommand {
    Close(Close),
    CreateUserContext(CreateUserContext),
    GetClientWindows(GetClientWindows),
    GetUserContexts(GetUserContexts),
    RemoveUserContext(RemoveUserContext),
    SetClientWindowState(SetClientWindowState),
    SetDownloadBehavior(SetDownloadBehavior),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum BrowserResult {
    CreateUserContextResult(CreateUserContextResult),
    GetClientWindowsResult(GetClientWindowsResult),
    GetUserContextsResult(GetUserContextsResult),
    SetClientWindowStateResult(SetClientWindowStateResult),
}

pub type ClientWindow = String;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ClientWindowInfo {
    pub active: bool,
    #[serde(rename = "clientWindow")]
    pub client_window: ClientWindow,
    pub height: JsUint,
    pub state: ClientWindowState,
    pub width: JsUint,
    pub x: JsInt,
    pub y: JsInt,
}

impl ClientWindowInfo {
    pub fn new(
        active: bool,
        client_window: ClientWindow,
        height: JsUint,
        state: ClientWindowState,
        width: JsUint,
        x: JsInt,
        y: JsInt,
    ) -> Self {
        Self {
            active,
            client_window,
            height,
            state,
            width,
            x,
            y,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ClientWindowState {
    Fullscreen,
    Maximized,
    Minimized,
    Normal,
}

pub type UserContext = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserContextInfo {
    #[serde(rename = "userContext")]
    pub user_context: UserContext,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Close {
    pub method: String,
    pub params: EmptyParams,
}

impl Close {
    pub fn new(params: EmptyParams) -> Self {
        Self {
            method: "browser.close".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateUserContext {
    pub method: String,
    pub params: CreateUserContextParameters,
}

impl CreateUserContext {
    pub fn new(params: CreateUserContextParameters) -> Self {
        Self {
            method: "browser.createUserContext".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateUserContextParameters {
    #[serde(
        rename = "acceptInsecureCerts",
        skip_serializing_if = "Option::is_none"
    )]
    pub accept_insecure_certs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyConfiguration>,
    #[serde(
        rename = "unhandledPromptBehavior",
        skip_serializing_if = "Option::is_none"
    )]
    pub unhandled_prompt_behavior: Option<UserPromptHandler>,
}

impl CreateUserContextParameters {
    pub fn new(
        accept_insecure_certs: Option<bool>,
        proxy: Option<ProxyConfiguration>,
        unhandled_prompt_behavior: Option<UserPromptHandler>,
    ) -> Self {
        Self {
            accept_insecure_certs,
            proxy,
            unhandled_prompt_behavior,
        }
    }
}

pub type CreateUserContextResult = UserContextInfo;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GetClientWindows {
    pub method: String,
    pub params: EmptyParams,
}

impl GetClientWindows {
    pub fn new(params: EmptyParams) -> Self {
        Self {
            method: "browser.getClientWindows".to_string(),
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetClientWindowsResult {
    #[serde(rename = "clientWindows")]
    pub client_windows: Vec<ClientWindowInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GetUserContexts {
    pub method: String,
    pub params: EmptyParams,
}

impl GetUserContexts {
    pub fn new(params: EmptyParams) -> Self {
        Self {
            method: "browser.getUserContexts".to_string(),
            params,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetUserContextsResult {
    #[serde(rename = "userContexts")]
    pub user_contexts: Vec<UserContextInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RemoveUserContext {
    pub method: String,
    pub params: RemoveUserContextParameters,
}

impl RemoveUserContext {
    pub fn new(params: RemoveUserContextParameters) -> Self {
        Self {
            method: "browser.removeUserContext".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RemoveUserContextParameters {
    #[serde(rename = "userContext")]
    pub user_context: UserContext,
}

impl RemoveUserContextParameters {
    pub fn new(user_context: UserContext) -> Self {
        Self { user_context }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SetClientWindowState {
    pub method: String,
    pub params: SetClientWindowStateParameters,
}

impl SetClientWindowState {
    pub fn new(params: SetClientWindowStateParameters) -> Self {
        Self {
            method: "browser.setClientWindowState".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SetClientWindowStateParameters {
    #[serde(rename = "clientWindow")]
    pub client_window: ClientWindow,
    /// Flattened into the parameters: the spec has no wrapper key here.
    #[serde(flatten)]
    pub client_window_named_state: ClientWindowNamedOrRectState,
}

impl SetClientWindowStateParameters {
    pub fn new(
        client_window: ClientWindow,
        client_window_named_state: ClientWindowNamedOrRectState,
    ) -> Self {
        Self {
            client_window,
            client_window_named_state,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ClientWindowNamedOrRectState {
    ClientWindowNamedState(ClientWindowNamedState),
    ClientWindowRectState(ClientWindowRectState),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ClientWindowNamedState {
    pub state: ClientWindowState,
}

impl ClientWindowNamedState {
    pub fn new(state: ClientWindowState) -> Self {
        Self { state }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ClientWindowRectState {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<JsUint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<JsUint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<JsInt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<JsInt>,
}

impl ClientWindowRectState {
    pub fn new(
        state: String,
        width: Option<JsUint>,
        height: Option<JsUint>,
        x: Option<JsInt>,
        y: Option<JsInt>,
    ) -> Self {
        Self {
            state,
            width,
            height,
            x,
            y,
        }
    }
}

pub type SetClientWindowStateResult = ClientWindowInfo;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SetDownloadBehavior {
    pub method: String,
    pub params: SetDownloadBehaviorParameters,
}

impl SetDownloadBehavior {
    pub fn new(params: SetDownloadBehaviorParameters) -> Self {
        Self {
            method: "browser.setDownloadBehavior".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SetDownloadBehaviorParameters {
    /// `None` resets the download behavior to the default.
    #[serde(rename = "downloadBehavior")]
    pub download_behavior: Option<DownloadBehavior>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetDownloadBehaviorParameters {
    pub fn new(
        download_behavior: Option<DownloadBehavior>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            download_behavior,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DownloadBehavior {
    Allowed {
        #[serde(rename = "destinationFolder")]
        destination_folder: String,
    },
    Denied,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn download_behavior_serialization() {
        let allowed = SetDownloadBehaviorParameters::new(
            Some(DownloadBehavior::Allowed {
                destination_folder: "/tmp".into(),
            }),
            None,
        );
        assert_eq!(
            serde_json::to_value(&allowed).unwrap(),
            json!({"downloadBehavior": {"type": "allowed", "destinationFolder": "/tmp"}})
        );

        let denied = SetDownloadBehaviorParameters::new(
            Some(DownloadBehavior::Denied),
            Some(vec!["u".into()]),
        );
        assert_eq!(
            serde_json::to_value(&denied).unwrap(),
            json!({"downloadBehavior": {"type": "denied"}, "userContexts": ["u"]})
        );

        let reset = SetDownloadBehaviorParameters::new(None, None);
        assert_eq!(
            serde_json::to_value(&reset).unwrap(),
            json!({"downloadBehavior": null})
        );
    }
}
