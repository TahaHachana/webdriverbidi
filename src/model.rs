use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub id: JsUint,
    pub command_data: CommandData,
    pub extensible: Extensible,
}

impl Command {
    pub fn new(id: JsUint, command_data: CommandData) -> Self {
        Self {
            id,
            command_data,
            extensible: Extensible::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommandData {
    BrowserCommand(BrowserCommand),
    BrowsingContextCommand(BrowsingContextCommand),
    InputCommand(InputCommand),
    NetworkCommand(NetworkCommand),
    ScriptCommand(ScriptCommand),
    SessionCommand(SessionCommand),
    StorageCommand(StorageCommand),
    WebExtensionCommand(WebExtensionCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyParams {
    pub extensible: Extensible,
}

impl EmptyParams {
    pub fn new() -> Self {
        Self {
            extensible: Extensible::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Message {
    CommandResponse(CommandResponse),
    ErrorResponse(ErrorResponse),
    Event(Event),
}

#[derive(Debug, Serialize, Deserialize)]
struct CommandResponse {
    #[serde(rename = "type")]
    response_type: String,
    id: JsUint,
    result: ResultData,
    #[serde(flatten)]
    extensible: Extensible,
}

impl CommandResponse {
    pub fn new(
        response_type: String,
        id: JsUint,
        result: ResultData,
        extensible: Extensible,
    ) -> Self {
        Self {
            response_type,
            id,
            result,
            extensible,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    #[serde(rename = "type")]
    response_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<JsUint>,
    error: ErrorCode,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stacktrace: Option<String>,
    #[serde(flatten)]
    extensible: Extensible,
}

impl ErrorResponse {
    pub fn new(
        response_type: String,
        id: Option<JsUint>,
        error: ErrorCode,
        message: String,
        stacktrace: Option<String>,
        extensible: Extensible,
    ) -> Self {
        Self {
            response_type,
            id,
            error,
            message,
            stacktrace,
            extensible,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ResultData {
    BrowsingContextResult(BrowsingContextResult),
    EmptyResult(EmptyResult),
    NetworkResult(NetworkResult),
    ScriptResult(ScriptResult),
    SessionResult(SessionResult),
    StorageResult(StorageResult),
    WebExtensionResult(WebExtensionResult),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmptyResult {
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    pub event_data: EventData,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EventData {
    BrowsingContextEvent(BrowsingContextEvent),
    LogEvent(LogEvent),
    NetworkEvent(NetworkEvent),
    ScriptEvent(ScriptEvent),
}

pub type Extensible = HashMap<String, String>;

// -9007199254740991..9007199254740991
pub type JsInt = i64;
// 0..9007199254740991
pub type JsUint = u64;


#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ErrorCode {
    #[serde(rename = "invalid argument")]
    InvalidArgument,
    #[serde(rename = "invalid selector")]
    InvalidSelector,
    #[serde(rename = "invalid session id")]
    InvalidSessionId,
    #[serde(rename = "invalid web extension")]
    InvalidWebExtension,
    #[serde(rename = "move target out of bounds")]
    MoveTargetOutOfBounds,
    #[serde(rename = "no such alert")]
    NoSuchAlert,
    #[serde(rename = "no such element")]
    NoSuchElement,
    #[serde(rename = "no such frame")]
    NoSuchFrame,
    #[serde(rename = "no such handle")]
    NoSuchHandle,
    #[serde(rename = "no such history entry")]
    NoSuchHistoryEntry,
    #[serde(rename = "no such intercept")]
    NoSuchIntercept,
    #[serde(rename = "no such node")]
    NoSuchNode,
    #[serde(rename = "no such request")]
    NoSuchRequest,
    #[serde(rename = "no such script")]
    NoSuchScript,
    #[serde(rename = "no such storage partition")]
    NoSuchStoragePartition,
    #[serde(rename = "no such user context")]
    NoSuchUserContext,
    #[serde(rename = "no such web extension")]
    NoSuchWebExtension,
    #[serde(rename = "session not created")]
    SessionNotCreated,
    #[serde(rename = "unable to capture screen")]
    UnableToCaptureScreen,
    #[serde(rename = "unable to close browser")]
    UnableToCloseBrowser,
    #[serde(rename = "unable to set cookie")]
    UnableToSetCookie,
    #[serde(rename = "unable to set file input")]
    UnableToSetFileInput,
    #[serde(rename = "underspecified storage partition")]
    UnderspecifiedStoragePartition,
    #[serde(rename = "unknown command")]
    UnknownCommand,
    #[serde(rename = "unknown error")]
    UnknownError,
    #[serde(rename = "unsupported operation")]
    UnsupportedOperation,
}

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

pub use session::*;

pub mod session {
    use super::*;

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
            Self {
                events,
                contexts,
            }
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BrowserCommand {
    Close(Close),
    CreateUserContext(CreateUserContext),
    GetClientWindows(GetClientWindows),
    GetUserContexts(GetUserContexts),
    RemoveUserContext(RemoveUserContext),
    SetClientWindowState(SetClientWindowState),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum BrowserResult {
    CreateUserContextResult(CreateUserContextResult),
    GetUserContextsResult(GetUserContextsResult),
}

pub use browser::*;

mod browser {
    use super::*;

    pub type ClientWindow = String;

    #[derive(Debug, Serialize, Deserialize)]
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
    
    pub type UserContext = String;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct UserContextInfo {
        #[serde(rename = "userContext")]
        pub user_context: UserContext,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
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
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CreateUserContext {
        pub method: String,
        pub params: EmptyParams,
    }
    
    impl CreateUserContext {
        pub fn new(params: EmptyParams) -> Self {
            Self {
                method: "browser.createUserContext".to_string(),
                params,
            }
        }
    }
    
    pub type CreateUserContextResult = UserContextInfo;
    
    #[derive(Debug, Serialize, Deserialize)]
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
    
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    pub struct GetClientWindowsResult {
        #[serde(rename = "clientWindows")]
        pub client_windows: Vec<ClientWindowInfo>,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
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
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct GetUserContextsResult {
        #[serde(rename = "userContexts")]
        pub user_contexts: Vec<UserContextInfo>,
    }

    #[derive(Debug, Serialize, Deserialize)]
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
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct RemoveUserContextParameters {
        #[serde(rename = "userContext")]
        pub user_context: UserContext,
    }
    
    impl RemoveUserContextParameters {
        pub fn new(user_context: UserContext) -> Self {
            Self { user_context }
        }
    }
    

    #[derive(Debug, Serialize, Deserialize)]
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
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SetClientWindowStateParameters {
        #[serde(rename = "clientWindow")]
        pub client_window: ClientWindow,
        #[serde(rename = "clientWindowNamedState")]
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
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum ClientWindowNamedOrRectState {
        ClientWindowNamedState(ClientWindowNamedState),
        ClientWindowRectState(ClientWindowRectState),
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ClientWindowNamedState {
        pub state: ClientWindowState,
    }
    
    impl ClientWindowNamedState {
        pub fn new(state: ClientWindowState) -> Self {
            Self { state }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BrowsingContextCommand {
    Activate(Activate),
    CaptureScreenshot(CaptureScreenshot),
    Close(Close),
    Create(Create),
    GetTree(GetTree),
    HandleUserPrompt(HandleUserPrompt),
    LocateNodes(LocateNodes),
    Navigate(Navigate),
    Print(Print),
    Reload(Reload),
    SetViewport(SetViewport),
    TraverseHistory(TraverseHistory),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum BrowsingContextResult {
    CaptureScreenshotResult(CaptureScreenshotResult),
    CreateResult(CreateResult),
    GetTreeResult(GetTreeResult),
    LocateNodesResult(LocateNodesResult),
    NavigateResult(NavigateResult),
    PrintResult(PrintResult),
    TraverseHistoryResult(TraverseHistoryResult),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum BrowsingContextEvent {
    ContextCreated(ContextCreated),
    ContextDestroyed(ContextDestroyed),
    DomContentLoaded(DomContentLoaded),
    DownloadWillBegin(DownloadWillBegin),
    FragmentNavigated(FragmentNavigated),
    HistoryUpdated(HistoryUpdated),
    Load(Load),
    NavigationAborted(NavigationAborted),
    NavigationCommitted(NavigationCommitted),
    NavigationFailed(NavigationFailed),
    NavigationStarted(NavigationStarted),
    UserPromptClosed(UserPromptClosed),
    UserPromptOpened(UserPromptOpened),
}

mod browsing_context {
    use super::*;

    pub type BrowsingContext = String;
    
    pub type InfoList = Vec<Info>;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Info {
        pub children: Option<InfoList>,
        #[serde(rename = "clientWindow", skip_serializing_if = "Option::is_none")]
        pub client_window: Option<browser::ClientWindow>,
        pub context: BrowsingContext,
        #[serde(rename = "originalOpener")]
        pub original_opener: Option<BrowsingContext>,
        pub url: String,
        #[serde(rename = "userContext")]
        pub user_context: browser::UserContext,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub parent: Option<BrowsingContext>,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Locator {
        AccessibilityLocator(AccessibilityLocator),
        CssLocator(CssLocator),
        ContextLocator(ContextLocator),
        InnerTextLocator(InnerTextLocator),
        XPathLocator(XPathLocator),
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AccessibilityLocator {
        #[serde(rename = "type")]
        pub locator_type: String,
        pub value: AccessibilityLocatorValue,
    }
    
    impl AccessibilityLocator {
        pub fn new(value: AccessibilityLocatorValue) -> Self {
            Self {
                locator_type: "accessibility".to_string(),
                value,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct AccessibilityLocatorValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub role: Option<String>,
    }
    
    impl AccessibilityLocatorValue {
        pub fn new(name: Option<String>, role: Option<String>) -> Self {
            Self { name, role }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CssLocator {
        #[serde(rename = "type")]
        pub locator_type: String,
        pub value: String,
    }
    
    impl CssLocator {
        pub fn new(value: String) -> Self {
            Self {
                locator_type: "css".to_string(),
                value,
            }
        }
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ContextLocator {
        #[serde(rename = "type")]
        pub locator_type: String,
        pub value: ContextValue,
    }
    
    impl ContextLocator {
        pub fn new(value: ContextValue) -> Self {
            Self {
                locator_type: "context".to_string(),
                value,
            }
        }
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ContextValue {
        pub context: BrowsingContext,
    }
    
    impl ContextValue {
        pub fn new(context: BrowsingContext) -> Self {
            Self { context }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct InnerTextLocator {
        #[serde(rename = "type")]
        pub locator_type: String,
        pub value: String,
        #[serde(rename = "ignoreCase", skip_serializing_if = "Option::is_none")]
        pub ignore_case: Option<bool>,
        #[serde(rename = "matchType", skip_serializing_if = "Option::is_none")]
        pub match_type: Option<InnerTextLocatorMatchType>,
        #[serde(rename = "maxDepth", skip_serializing_if = "Option::is_none")]
        pub max_depth: Option<JsUint>,
    }
    
    impl InnerTextLocator {
        pub fn new(
            value: String,
            ignore_case: Option<bool>,
            match_type: Option<InnerTextLocatorMatchType>,
            max_depth: Option<JsUint>,
        ) -> Self {
            Self {
                locator_type: "innerText".to_string(),
                value,
                ignore_case,
                match_type,
                max_depth,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum InnerTextLocatorMatchType {
        Full,
        Partial,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct XPathLocator {
        #[serde(rename = "type")]
        pub locator_type: String,
        pub value: String,
    }
    
    impl XPathLocator {
        pub fn new(value: String) -> Self {
            Self {
                locator_type: "xpath".to_string(),
                value,
            }
        }
    }
    
    pub type Navigation = String;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct BaseNavigationInfo {
        pub context: BrowsingContext,
        pub navigation: Option<Navigation>,
        pub timestamp: JsUint,
        pub url: String,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct NavigationInfo {
        #[serde(flatten)]
        pub base: BaseNavigationInfo,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum ReadinessState {
        Complete,
        Interactive,
        None,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum UserPromptType {
        Alert,
        BeforeUnload,
        Confirm,
        Prompt,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Activate {
        pub method: String,
        pub params: ActivateParameters,
    }
    
    impl Activate {
        pub fn new(params: ActivateParameters) -> Self {
            Self {
                method: "browsingContext.activate".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ActivateParameters {
        pub context: BrowsingContext,
    }
    
    impl ActivateParameters {
        pub fn new(context: BrowsingContext) -> Self {
            Self { context }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CaptureScreenshot {
        pub method: String,
        pub params: CaptureScreenshotParameters,
    }
    
    impl CaptureScreenshot {
        pub fn new(params: CaptureScreenshotParameters) -> Self {
            Self {
                method: "browsingContext.captureScreenshot".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CaptureScreenshotParameters {
        pub context: BrowsingContext,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub origin: Option<CaptureScreenshotParametersOrigin>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub format: Option<ImageFormat>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub clip: Option<ClipRectangle>,
    }
    
    impl CaptureScreenshotParameters {
        pub fn new(
            context: BrowsingContext,
            origin: Option<CaptureScreenshotParametersOrigin>,
            format: Option<ImageFormat>,
            clip: Option<ClipRectangle>,
        ) -> Self {
            Self {
                context,
                origin,
                format,
                clip,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum CaptureScreenshotParametersOrigin {
        Document,
        Viewport,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ImageFormat {
        #[serde(rename = "type")]
        pub image_format_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub quality: Option<f32>, // 0.0..1.0
    }
    
    impl ImageFormat {
        pub fn new(image_format_type: String, quality: Option<f32>) -> Self {
            Self {
                image_format_type,
                quality,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum ClipRectangle {
        BoxClipRectangle(BoxClipRectangle),
        ElementClipRectangle(ElementClipRectangle),
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ElementClipRectangle {
        #[serde(rename = "type")]
        pub clip_rectangle_type: String,
        pub element: SharedReference,
    }
    
    impl ElementClipRectangle {
        pub fn new(element: SharedReference) -> Self {
            Self {
                clip_rectangle_type: "element".to_string(),
                element,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct BoxClipRectangle {
        #[serde(rename = "type")]
        pub clip_rectangle_type: String,
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
    }
    
    impl BoxClipRectangle {
        pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
            Self {
                clip_rectangle_type: "box".to_string(),
                x,
                y,
                width,
                height,
            }
        }
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct CaptureScreenshotResult {
        pub data: String,
    }
    

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Close {
        pub method: String,
        pub params: CloseParameters,
    }
    
    impl Close {
        pub fn new(params: CloseParameters) -> Self {
            Self {
                method: "browsingContext.close".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CloseParameters {
        pub context: BrowsingContext,
        #[serde(rename = "promptUnload", skip_serializing_if = "Option::is_none")]
        pub prompt_unload: Option<bool>,
    }
    
    impl CloseParameters {
        pub fn new(context: BrowsingContext, prompt_unload: Option<bool>) -> Self {
            Self {
                context,
                prompt_unload,
            }
        }
    }
    

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Create {
        pub method: String,
        pub params: CreateParameters,
    }
    
    impl Create {
        pub fn new(params: CreateParameters) -> Self {
            Self {
                method: "browsingContext.create".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum CreateType {
        Tab,
        Window,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CreateParameters {
        #[serde(rename = "type")]
        pub create_type: CreateType,
        #[serde(rename = "referenceContext", skip_serializing_if = "Option::is_none")]
        pub reference_context: Option<BrowsingContext>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub background: Option<bool>,
        #[serde(rename = "userContext", skip_serializing_if = "Option::is_none")]
        pub user_context: Option<browser::UserContext>,
    }
    
    impl CreateParameters {
        pub fn new(
            create_type: CreateType,
            reference_context: Option<BrowsingContext>,
            background: Option<bool>,
            user_context: Option<browser::UserContext>,
        ) -> Self {
            Self {
                create_type,
                reference_context,
                background,
                user_context,
            }
        }
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct CreateResult {
        pub context: BrowsingContext,
    }


    #[derive(Debug, Serialize, Deserialize)]
    pub struct GetTree {
        pub method: String,
        pub params: GetTreeParameters,
    }
    
    impl GetTree {
        pub fn new(params: GetTreeParameters) -> Self {
            Self {
                method: "browsingContext.getTree".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct GetTreeParameters {
        #[serde(rename = "maxDepth", skip_serializing_if = "Option::is_none")]
        pub max_depth: Option<JsUint>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub root: Option<BrowsingContext>,
    }
    
    impl GetTreeParameters {
        pub fn new(max_depth: Option<JsUint>, root: Option<BrowsingContext>) -> Self {
            Self { max_depth, root }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct GetTreeResult {
        pub contexts: InfoList,
    }


    #[derive(Debug, Serialize, Deserialize)]
    pub struct HandleUserPrompt {
        pub method: String,
        pub params: HandleUserPromptParameters,
    }
    
    impl HandleUserPrompt {
        pub fn new(params: HandleUserPromptParameters) -> Self {
            Self {
                method: "browsingContext.handleUserPrompt".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct HandleUserPromptParameters {
        pub context: BrowsingContext,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub accept: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_text: Option<String>,
    }
    
    impl HandleUserPromptParameters {
        pub fn new(context: BrowsingContext, accept: Option<bool>, user_text: Option<String>) -> Self {
            Self {
                context,
                accept,
                user_text,
            }
        }
    }
    

    #[derive(Debug, Serialize, Deserialize)]
    pub struct LocateNodes {
        pub method: String,
        pub params: LocateNodesParameters,
    }
    
    impl LocateNodes {
        pub fn new(params: LocateNodesParameters) -> Self {
            Self {
                method: "browsingContext.locateNodes".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct LocateNodesParameters {
        pub context: BrowsingContext,
        pub locator: Locator,
        #[serde(rename = "maxNodeCount", skip_serializing_if = "Option::is_none")]
        pub max_node_count: Option<JsUint>,
        #[serde(
            rename = "serializationOptions",
            skip_serializing_if = "Option::is_none"
        )]
        pub serialization_options: Option<SerializationOptions>,
        #[serde(rename = "startNodes", skip_serializing_if = "Option::is_none")]
        pub start_nodes: Option<Vec<SharedReference>>,
    }
    
    impl LocateNodesParameters {
        pub fn new(
            context: BrowsingContext,
            locator: Locator,
            max_node_count: Option<JsUint>,
            serialization_options: Option<SerializationOptions>,
            start_nodes: Option<Vec<SharedReference>>,
        ) -> Self {
            Self {
                context,
                locator,
                max_node_count,
                serialization_options,
                start_nodes,
            }
        }
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct LocateNodesResult {
        pub nodes: Vec<script::NodeRemoteValue>,
    }
    

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Navigate {
        pub method: String, // "browsingContext.navigate"
        pub params: NavigateParameters,
    }
    
    impl Navigate {
        pub fn new(params: NavigateParameters) -> Self {
            Self {
                method: "browsingContext.navigate".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct NavigateParameters {
        pub context: BrowsingContext,
        pub url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub wait: Option<ReadinessState>,
    }
    
    impl NavigateParameters {
        pub fn new(context: BrowsingContext, url: String, wait: Option<ReadinessState>) -> Self {
            Self { context, url, wait }
        }
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct NavigateResult {
        pub navigation: Option<Navigation>,
        pub url: String,
    }


    #[derive(Debug, Serialize, Deserialize)]
    pub struct Print {
        pub method: String,
        pub params: PrintParameters,
    }
    
    impl Print {
        pub fn new(params: PrintParameters) -> Self {
            Self {
                method: "browsingContext.print".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct PrintParameters {
        pub context: BrowsingContext,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub background: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub margin: Option<PrintMarginParameters>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub orientation: Option<PrintParametersOrientation>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub page: Option<PrintPageParameters>,
        #[serde(rename = "pageRanges", skip_serializing_if = "Option::is_none")]
        pub page_ranges: Option<Vec<JsUintOrText>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub scale: Option<f32>, // 0.1..2.0
        #[serde(rename = "shrinkToFit", skip_serializing_if = "Option::is_none")]
        pub shrink_to_fit: Option<bool>,
    }
    
    impl PrintParameters {
        pub fn new(
            context: BrowsingContext,
            background: Option<bool>,
            margin: Option<PrintMarginParameters>,
            orientation: Option<PrintParametersOrientation>,
            page: Option<PrintPageParameters>,
            page_ranges: Option<Vec<JsUintOrText>>,
            scale: Option<f32>,
            shrink_to_fit: Option<bool>,
        ) -> Self {
            Self {
                context,
                background,
                margin,
                orientation,
                page,
                page_ranges,
                scale,
                shrink_to_fit,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum PrintParametersOrientation {
        Landscape,
        Portrait,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum JsUintOrText {
        JsUint(JsUint),
        Text(String),
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct PrintMarginParameters {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bottom: Option<f32>, // 0.0..
        #[serde(skip_serializing_if = "Option::is_none")]
        pub left: Option<f32>, // 0.0..
        #[serde(skip_serializing_if = "Option::is_none")]
        pub right: Option<f32>, // 0.0..
        #[serde(skip_serializing_if = "Option::is_none")]
        pub top: Option<f32>, // 0.0..
    }
    
    impl PrintMarginParameters {
        pub fn new(
            bottom: Option<f32>,
            left: Option<f32>,
            right: Option<f32>,
            top: Option<f32>,
        ) -> Self {
            Self {
                bottom,
                left,
                right,
                top,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct PrintPageParameters {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub height: Option<f32>, // 0.0352..
        #[serde(skip_serializing_if = "Option::is_none")]
        pub width: Option<f32>, // 0.0352..
    }
    
    impl PrintPageParameters {
        pub fn new(height: Option<f32>, width: Option<f32>) -> Self {
            Self { height, width }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct PrintResult {
        pub data: String,
    }


    #[derive(Debug, Serialize, Deserialize)]
    pub struct Reload {
        pub method: String,
        pub params: ReloadParameters,
    }
    
    impl Reload {
        pub fn new(params: ReloadParameters) -> Self {
            Self {
                method: "browsingContext.reload".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ReloadParameters {
        pub context: BrowsingContext,
        #[serde(rename = "ignoreCache", skip_serializing_if = "Option::is_none")]
        pub ignore_cache: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub wait: Option<ReadinessState>,
    }
    
    impl ReloadParameters {
        pub fn new(
            context: BrowsingContext,
            ignore_cache: Option<bool>,
            wait: Option<ReadinessState>,
        ) -> Self {
            Self {
                context,
                ignore_cache,
                wait,
            }
        }
    }


    #[derive(Debug, Serialize, Deserialize)]
    pub struct SetViewport {
        pub method: String,
        pub params: SetViewportParameters,
    }
    
    impl SetViewport {
        pub fn new(params: SetViewportParameters) -> Self {
            Self {
                method: "browsingContext.setViewport".to_string(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SetViewportParameters {
        pub context: BrowsingContext,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub viewport: Option<Viewport>,
        #[serde(rename = "devicePixelRatio", skip_serializing_if = "Option::is_none")]
        pub device_pixel_ratio: Option<f32>, // 0.0..
    }
    
    impl SetViewportParameters {
        pub fn new(
            context: BrowsingContext,
            viewport: Option<Viewport>,
            device_pixel_ratio: Option<f32>,
        ) -> Self {
            Self {
                context,
                viewport,
                device_pixel_ratio,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Viewport {
        pub width: JsUint,
        pub height: JsUint,
    }
    
    impl Viewport {
        pub fn new(width: JsUint, height: JsUint) -> Self {
            Self { width, height }
        }
    }


    #[derive(Debug, Serialize, Deserialize)]
    pub struct TraverseHistory {
        pub method: String,
        pub params: TraverseHistoryParameters,
    }
    
    impl TraverseHistory {
        pub fn new(params: TraverseHistoryParameters) -> Self {
            Self {
                method: "browsingContext.traverseHistory".into(),
                params,
            }
        }
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct TraverseHistoryParameters {
        pub context: BrowsingContext,
        pub delta: JsInt,
    }
    
    impl TraverseHistoryParameters {
        pub fn new(context: String, delta: i64) -> Self {
            Self { context, delta }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct TraverseHistoryResult {}

    #[derive(Serialize, Deserialize, Debug)]
    pub struct ContextCreated {
        pub method: String,
        pub params: Info,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ContextDestroyed {
        pub method: String,
        pub params: Info,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct NavigationStarted {
        pub method: String,
        pub params: NavigationInfo,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct FragmentNavigated {
        pub method: String,
        pub params: NavigationInfo,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct HistoryUpdated {
        pub method: String,
        pub params: HistoryUpdatedParameters,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct HistoryUpdatedParameters {
        pub context: BrowsingContext,
        pub url: String,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct DomContentLoaded {
        pub method: String,
        pub params: NavigationInfo,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Load {
        pub method: String,
        pub params: NavigationInfo,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct DownloadWillBegin {
        pub method: String,
        pub params: NavigationInfo,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct NavigationAborted {
        pub method: String,
        pub params: NavigationInfo,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct NavigationCommitted {
        pub method: String,
        pub params: NavigationInfo,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct NavigationFailed {
        pub method: String,
        pub params: NavigationInfo,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct UserPromptClosed {
        pub method: String,
        pub params: UserPromptClosedParameters,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct UserPromptClosedParameters {
        pub context: BrowsingContext,
        pub accepted: bool,
        #[serde(rename = "type")]
        pub prompt_type: UserPromptType,
        #[serde(rename = "userText", skip_serializing_if = "Option::is_none")]
        pub user_text: Option<String>,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct UserPromptOpened {
        pub method: String,
        pub params: UserPromptOpenedParameters,
    }
    
    #[derive(Serialize, Deserialize, Debug)]
    pub struct UserPromptOpenedParameters {
        pub context: BrowsingContext,
        pub handler: session::UserPromptHandlerType,
        pub message: String,
        #[serde(rename = "type")]
        pub prompt_type: UserPromptType,
        #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
        pub default_value: Option<String>,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EmulationCommand {
    SetGeolocationOverride(SetGeolocationOverride),
}

pub mod emulation {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SetGeolocationOverride {
        pub method: String,
        pub params: SetGeolocationOverrideParameters,
    }
    
    impl SetClientWindowState {
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
        user_contexts: Option<Vec<userContexts>>,
    }

    impl SetGeolocationOverrideParameters {
        pub fn new(coordinates: Option<GeolocationCoordinates>, contexts: Option<Vec<BrowsingContext>>, user_contexts: Option<Vec<userContexts>>) -> Self {
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
    
        #[serde(default)]
        pub altitudeAccuracy: Option<f64>,

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
                altitudeAccuracy: None,
                heading: None,
                speed: None,
            }
        }
    
        fn default_accuracy() -> f64 {
            1.0
        }
    }    

}