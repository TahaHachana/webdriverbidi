use serde::{Deserialize, Serialize};

use crate::model::browser::{ClientWindow, UserContext};
use crate::model::common::{JsInt, JsUint};
use crate::model::script::{NodeRemoteValue, SerializationOptions, SharedReference};
use crate::model::session::UserPromptHandlerType;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    SetBypassCSP(SetBypassCSP),
    SetViewport(SetViewport),
    StartScreencast(StartScreencast),
    StopScreencast(StopScreencast),
    TraverseHistory(TraverseHistory),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum BrowsingContextResult {
    CaptureScreenshotResult(CaptureScreenshotResult),
    CreateResult(CreateResult),
    GetTreeResult(GetTreeResult),
    LocateNodesResult(LocateNodesResult),
    NavigateResult(NavigateResult),
    PrintResult(PrintResult),
    StartScreencastResult(StartScreencastResult),
    StopScreencastResult(StopScreencastResult),
    TraverseHistoryResult(TraverseHistoryResult),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum BrowsingContextEvent {
    ContextCreated(ContextCreated),
    ContextDestroyed(ContextDestroyed),
    DomContentLoaded(DomContentLoaded),
    DownloadEnd(DownloadEnd),
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

pub type BrowsingContext = String;

pub type InfoList = Vec<Info>;

/// `browsingContext.BaseInfo`: the fields shared by `Info` and `ContextCreatedParameters`.
pub type BaseInfo = Info;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Info {
    pub children: Option<InfoList>,
    #[serde(rename = "clientWindow", skip_serializing_if = "Option::is_none")]
    pub client_window: Option<ClientWindow>,
    pub context: BrowsingContext,
    #[serde(rename = "originalOpener")]
    pub original_opener: Option<BrowsingContext>,
    pub url: String,
    #[serde(rename = "userContext")]
    pub user_context: UserContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<BrowsingContext>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Locator {
    AccessibilityLocator(AccessibilityLocator),
    CssLocator(CssLocator),
    ContextLocator(ContextLocator),
    InnerTextLocator(InnerTextLocator),
    XPathLocator(XPathLocator),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ContextValue {
    pub context: BrowsingContext,
}

impl ContextValue {
    pub fn new(context: BrowsingContext) -> Self {
        Self { context }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InnerTextLocatorMatchType {
    Full,
    Partial,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

pub type Download = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BaseNavigationInfo {
    pub context: BrowsingContext,
    pub navigation: Option<Navigation>,
    pub timestamp: JsUint,
    pub url: String,
    #[serde(rename = "userContext", skip_serializing_if = "Option::is_none")]
    pub user_context: Option<UserContext>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NavigationInfo {
    #[serde(flatten)]
    pub base: BaseNavigationInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReadinessState {
    Complete,
    Interactive,
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserPromptType {
    Alert,
    BeforeUnload,
    Confirm,
    Prompt,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ActivateParameters {
    pub context: BrowsingContext,
}

impl ActivateParameters {
    pub fn new(context: BrowsingContext) -> Self {
        Self { context }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CaptureScreenshotParameters {
    pub context: BrowsingContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<CaptureScreenshotParametersOrigin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ImageFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip: Option<ClipRectangle>,
    #[serde(rename = "imageSize", skip_serializing_if = "Option::is_none")]
    pub image_size: Option<ImageSize>,
}

impl CaptureScreenshotParameters {
    pub fn new(
        context: BrowsingContext,
        origin: Option<CaptureScreenshotParametersOrigin>,
        format: Option<ImageFormat>,
        clip: Option<ClipRectangle>,
        image_size: Option<ImageSize>,
    ) -> Self {
        Self {
            context,
            origin,
            format,
            clip,
            image_size,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ImageSize {
    #[serde(rename = "maxWidth", skip_serializing_if = "Option::is_none")]
    pub max_width: Option<JsUint>, // >= 1
    #[serde(rename = "maxHeight", skip_serializing_if = "Option::is_none")]
    pub max_height: Option<JsUint>, // >= 1
}

impl ImageSize {
    pub fn new(max_width: Option<JsUint>, max_height: Option<JsUint>) -> Self {
        Self {
            max_width,
            max_height,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CaptureScreenshotParametersOrigin {
    Document,
    Viewport,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum ClipRectangle {
    BoxClipRectangle(BoxClipRectangle),
    ElementClipRectangle(ElementClipRectangle),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CaptureScreenshotResult {
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CreateType {
    Tab,
    Window,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateParameters {
    #[serde(rename = "type")]
    pub create_type: CreateType,
    #[serde(rename = "referenceContext", skip_serializing_if = "Option::is_none")]
    pub reference_context: Option<BrowsingContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    #[serde(rename = "userContext", skip_serializing_if = "Option::is_none")]
    pub user_context: Option<UserContext>,
}

impl CreateParameters {
    pub fn new(
        create_type: CreateType,
        reference_context: Option<BrowsingContext>,
        background: Option<bool>,
        user_context: Option<UserContext>,
    ) -> Self {
        Self {
            create_type,
            reference_context,
            background,
            user_context,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateResult {
    pub context: BrowsingContext,
    #[serde(rename = "userContext", skip_serializing_if = "Option::is_none")]
    pub user_context: Option<UserContext>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GetTreeResult {
    pub contexts: InfoList,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HandleUserPromptParameters {
    pub context: BrowsingContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept: Option<bool>,
    #[serde(rename = "userText", skip_serializing_if = "Option::is_none")]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LocateNodesResult {
    pub nodes: Vec<NodeRemoteValue>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NavigateResult {
    pub navigation: Option<Navigation>,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
    #[allow(
        clippy::too_many_arguments,
        reason = "one argument per spec parameter; struct literals are the alternative"
    )]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PrintParametersOrientation {
    Landscape,
    Portrait,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum JsUintOrText {
    JsUint(JsUint),
    Text(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PrintResult {
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SetViewportParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<BrowsingContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,
    #[serde(rename = "devicePixelRatio", skip_serializing_if = "Option::is_none")]
    pub device_pixel_ratio: Option<f32>, // 0.0..
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetViewportParameters {
    pub fn new(
        context: Option<BrowsingContext>,
        viewport: Option<Viewport>,
        device_pixel_ratio: Option<f32>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            context,
            viewport,
            device_pixel_ratio,
            user_contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Viewport {
    pub width: JsUint,
    pub height: JsUint,
}

impl Viewport {
    pub fn new(width: JsUint, height: JsUint) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TraverseHistoryParameters {
    pub context: BrowsingContext,
    pub delta: JsInt,
}

impl TraverseHistoryParameters {
    pub fn new(context: String, delta: i64) -> Self {
        Self { context, delta }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TraverseHistoryResult {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SetBypassCSP {
    pub method: String,
    pub params: SetBypassCSPParameters,
}

impl SetBypassCSP {
    pub fn new(params: SetBypassCSPParameters) -> Self {
        Self {
            method: "browsingContext.setBypassCSP".into(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SetBypassCSPParameters {
    /// `Some(true)` bypasses CSP, `None` restores the default behavior.
    pub bypass: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(rename = "userContexts", skip_serializing_if = "Option::is_none")]
    pub user_contexts: Option<Vec<UserContext>>,
}

impl SetBypassCSPParameters {
    pub fn new(
        bypass: Option<bool>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
    ) -> Self {
        Self {
            bypass,
            contexts,
            user_contexts,
        }
    }
}

pub type Screencast = String;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StartScreencast {
    pub method: String,
    pub params: StartScreencastParameters,
}

impl StartScreencast {
    pub fn new(params: StartScreencastParameters) -> Self {
        Self {
            method: "browsingContext.startScreencast".into(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StartScreencastParameters {
    pub context: BrowsingContext,
    #[serde(rename = "destinationFolder", skip_serializing_if = "Option::is_none")]
    pub destination_folder: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<MediaTrackConstraints>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<bool>,
}

impl StartScreencastParameters {
    pub fn new(
        context: BrowsingContext,
        destination_folder: Option<String>,
        mime_type: Option<String>,
        video: Option<MediaTrackConstraints>,
        audio: Option<bool>,
    ) -> Self {
        Self {
            context,
            destination_folder,
            mime_type,
            video,
            audio,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct MediaTrackConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<JsUint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<JsUint>,
    #[serde(rename = "frameRate", skip_serializing_if = "Option::is_none")]
    pub frame_rate: Option<JsUint>,
}

impl MediaTrackConstraints {
    pub fn new(width: Option<JsUint>, height: Option<JsUint>, frame_rate: Option<JsUint>) -> Self {
        Self {
            width,
            height,
            frame_rate,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StartScreencastResult {
    pub screencast: Screencast,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StopScreencast {
    pub method: String,
    pub params: StopScreencastParameters,
}

impl StopScreencast {
    pub fn new(params: StopScreencastParameters) -> Self {
        Self {
            method: "browsingContext.stopScreencast".into(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct StopScreencastParameters {
    pub screencast: Screencast,
}

impl StopScreencastParameters {
    pub fn new(screencast: Screencast) -> Self {
        Self { screencast }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StopScreencastResult {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ContextCreated {
    pub method: String,
    pub params: ContextCreatedParameters,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ContextCreatedParameters {
    #[serde(flatten)]
    pub info: BaseInfo,
    /// Added to the spec in 2026; `None` when the browser doesn't send it yet
    /// (e.g. Chrome 158).
    #[serde(
        rename = "hasPlannedNavigation",
        skip_serializing_if = "Option::is_none"
    )]
    pub has_planned_navigation: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ContextDestroyed {
    pub method: String,
    pub params: Info,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NavigationStarted {
    pub method: String,
    pub params: NavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FragmentNavigated {
    pub method: String,
    pub params: NavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HistoryUpdated {
    pub method: String,
    pub params: HistoryUpdatedParameters,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HistoryUpdatedParameters {
    pub context: BrowsingContext,
    pub timestamp: JsUint,
    pub url: String,
    #[serde(rename = "userContext", skip_serializing_if = "Option::is_none")]
    pub user_context: Option<UserContext>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DomContentLoaded {
    pub method: String,
    pub params: NavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Load {
    pub method: String,
    pub params: NavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DownloadWillBegin {
    pub method: String,
    pub params: DownloadWillBeginParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DownloadWillBeginParams {
    /// `None` when the browser predates this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<Download>,
    #[serde(rename = "suggestedFilename")]
    pub suggested_filename: String,
    #[serde(flatten)]
    pub base: BaseNavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DownloadEnd {
    pub method: String,
    pub params: DownloadEndParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum DownloadEndParams {
    DownloadCanceled(DownloadCanceledParams),
    DownloadComplete(DownloadCompleteParams),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DownloadCanceledParams {
    #[serde(deserialize_with = "crate::model::common::tag::canceled")]
    pub status: String,
    /// `None` when the browser predates this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<Download>,
    #[serde(flatten)]
    pub base: BaseNavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DownloadCompleteParams {
    #[serde(deserialize_with = "crate::model::common::tag::complete")]
    pub status: String,
    /// `None` when the browser predates this field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<Download>,
    pub filepath: Option<String>,
    #[serde(flatten)]
    pub base: BaseNavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NavigationAborted {
    pub method: String,
    pub params: NavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NavigationCommitted {
    pub method: String,
    pub params: NavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NavigationFailed {
    pub method: String,
    pub params: NavigationInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserPromptClosed {
    pub method: String,
    pub params: UserPromptClosedParameters,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserPromptClosedParameters {
    pub context: BrowsingContext,
    pub accepted: bool,
    #[serde(rename = "type")]
    pub prompt_type: UserPromptType,
    #[serde(rename = "userContext", skip_serializing_if = "Option::is_none")]
    pub user_context: Option<UserContext>,
    #[serde(rename = "userText", skip_serializing_if = "Option::is_none")]
    pub user_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserPromptOpened {
    pub method: String,
    pub params: UserPromptOpenedParameters,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserPromptOpenedParameters {
    pub context: BrowsingContext,
    pub handler: UserPromptHandlerType,
    pub message: String,
    #[serde(rename = "type")]
    pub prompt_type: UserPromptType,
    #[serde(rename = "userContext", skip_serializing_if = "Option::is_none")]
    pub user_context: Option<UserContext>,
    #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn context_created_parameters_flatten_base_info() {
        let raw = json!({
            "children": null,
            "clientWindow": "w1",
            "context": "c1",
            "originalOpener": null,
            "url": "about:blank",
            "userContext": "default",
            "hasPlannedNavigation": true
        });
        let params: ContextCreatedParameters = serde_json::from_value(raw).unwrap();
        assert_eq!(params.info.context, "c1");
        assert_eq!(params.info.user_context, "default");
        assert_eq!(params.has_planned_navigation, Some(true));
    }

    #[test]
    fn navigation_info_optional_user_context() {
        let raw = json!({"context": "c", "navigation": null, "timestamp": 1, "url": "u"});
        let info: NavigationInfo = serde_json::from_value(raw).unwrap();
        assert!(info.base.user_context.is_none());
    }

    #[test]
    fn capture_screenshot_image_size() {
        let p = CaptureScreenshotParameters::new(
            "c".into(),
            None,
            None,
            None,
            Some(ImageSize::new(Some(100), None)),
        );
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            json!({"context": "c", "imageSize": {"maxWidth": 100}})
        );
    }

    #[test]
    fn set_bypass_csp_reset_serializes_null() {
        let p = SetBypassCSPParameters::new(None, None, None);
        assert_eq!(serde_json::to_value(&p).unwrap(), json!({"bypass": null}));
    }
}
