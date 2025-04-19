use serde::{Deserialize, Serialize};

use crate::model::browser::{ClientWindow, UserContext};
use crate::model::common::{JsInt, JsUint};
use crate::model::script::{NodeRemoteValue, SerializationOptions, SharedReference};
use crate::model::session::UserPromptHandlerType;

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

pub type BrowsingContext = String;

pub type InfoList = Vec<Info>;

#[derive(Serialize, Deserialize, Debug)]
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
    pub nodes: Vec<NodeRemoteValue>,
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
    pub params: DownloadWillBeginParams,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadWillBeginParams {
    #[serde(rename = "suggestedFilename")]
    pub suggested_filename: String,
    #[serde(flatten)]
    pub base: BaseNavigationInfo,
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
    pub handler: UserPromptHandlerType,
    pub message: String,
    #[serde(rename = "type")]
    pub prompt_type: UserPromptType,
    #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
}
