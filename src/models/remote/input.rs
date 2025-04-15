use crate::remote::browsing_context::BrowsingContext;
use crate::remote::script::SharedReference;
use crate::remote::{JsInt, JsUint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputCommand {
    PerformActions(PerformActions),
    ReleaseActions(ReleaseActions),
    SetFiles(SetFiles),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementOrigin {
    #[serde(rename = "type")]
    pub element_origin_type: String,
    pub element: SharedReference,
}

impl ElementOrigin {
    pub fn new(element: SharedReference) -> Self {
        Self {
            element_origin_type: "element".to_string(),
            element,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformActions {
    pub method: String,
    pub params: PerformActionsParameters,
}

impl PerformActions {
    pub fn new(params: PerformActionsParameters) -> Self {
        Self {
            method: "input.performActions".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformActionsParameters {
    pub context: BrowsingContext,
    pub actions: Vec<SourceActions>,
}

impl PerformActionsParameters {
    pub fn new(context: BrowsingContext, actions: Vec<SourceActions>) -> Self {
        Self { context, actions }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SourceActions {
    NoneSourceActions(NoneSourceActions),
    KeySourceActions(KeySourceActions),
    PointerSourceActions(PointerSourceActions),
    WheelSourceActions(WheelSourceActions),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NoneSourceActions {
    #[serde(rename = "type")]
    pub none_source_actions_type: String,
    pub id: String,
    pub actions: Vec<NoneSourceAction>,
}

impl NoneSourceActions {
    pub fn new(id: String, actions: Vec<NoneSourceAction>) -> Self {
        Self {
            none_source_actions_type: "none".to_string(),
            id,
            actions,
        }
    }
}

pub type NoneSourceAction = PauseAction;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeySourceActions {
    #[serde(rename = "type")]
    pub key_source_actions_type: String,
    pub id: String,
    pub actions: Vec<KeySourceAction>,
}

impl KeySourceActions {
    pub fn new(id: String, actions: Vec<KeySourceAction>) -> Self {
        Self {
            key_source_actions_type: "key".to_string(),
            id,
            actions,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeySourceAction {
    PauseAction(PauseAction),
    KeyDownAction(KeyDownAction),
    KeyUpAction(KeyUpAction),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointerSourceActions {
    #[serde(rename = "type")]
    pub pointer_source_actions_type: String,
    pub id: String,
    pub parameters: Option<PointerParameters>,
    pub actions: Vec<PointerSourceAction>,
}

impl PointerSourceActions {
    pub fn new(
        id: String,
        parameters: Option<PointerParameters>,
        actions: Vec<PointerSourceAction>,
    ) -> Self {
        Self {
            pointer_source_actions_type: "pointer".to_string(),
            id,
            parameters,
            actions,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointerParameters {
    #[serde(rename = "pointerType", skip_serializing_if = "Option::is_none")]
    pub pointer_type: Option<PointerType>,
}

impl PointerParameters {
    pub fn new(pointer_type: Option<PointerType>) -> Self {
        Self { pointer_type }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PointerSourceAction {
    PauseAction(PauseAction),
    PointerDownAction(PointerDownAction),
    PointerUpAction(PointerUpAction),
    PointerMoveAction(PointerMoveAction),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WheelSourceActions {
    #[serde(rename = "type")]
    pub wheel_source_actions_type: String,
    pub id: String,
    pub actions: Vec<WheelSourceAction>,
}

impl WheelSourceActions {
    pub fn new(id: String, actions: Vec<WheelSourceAction>) -> Self {
        Self {
            wheel_source_actions_type: "wheel".to_string(),
            id,
            actions,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WheelSourceAction {
    PauseAction(PauseAction),
    WheelScrollAction(WheelScrollAction),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PauseAction {
    #[serde(rename = "type")]
    pub pause_action_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<JsUint>,
}

impl PauseAction {
    pub fn new(duration: Option<JsUint>) -> Self {
        Self {
            pause_action_type: "pause".to_string(),
            duration,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyDownAction {
    #[serde(rename = "type")]
    pub key_down_action_type: String,
    pub value: String,
}

impl KeyDownAction {
    pub fn new(value: String) -> Self {
        Self {
            key_down_action_type: "keyDown".to_string(),
            value,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyUpAction {
    #[serde(rename = "type")]
    pub key_up_action_type: String,
    pub value: String,
}

impl KeyUpAction {
    pub fn new(value: String) -> Self {
        Self {
            key_up_action_type: "keyUp".to_string(),
            value,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointerUpAction {
    #[serde(rename = "type")]
    pub pointer_up_action_type: String,
    pub button: JsUint,
}

impl PointerUpAction {
    pub fn new(button: JsUint) -> Self {
        Self {
            pointer_up_action_type: "pointerUp".to_string(),
            button,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointerDownAction {
    #[serde(rename = "type")]
    pub pointer_down_action_type: String,
    pub button: JsUint,
    #[serde(flatten)]
    pub pointer_common_properties: PointerCommonProperties,
}

impl PointerDownAction {
    pub fn new(button: JsUint, pointer_common_properties: PointerCommonProperties) -> Self {
        Self {
            pointer_down_action_type: "pointerDown".to_string(),
            button,
            pointer_common_properties,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointerMoveAction {
    #[serde(rename = "type")]
    pub pointer_move_action_type: String,
    pub x: JsInt,
    pub y: JsInt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<JsUint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    #[serde(flatten)]
    pub pointer_common_properties: PointerCommonProperties,
}

impl PointerMoveAction {
    pub fn new(
        x: JsInt,
        y: JsInt,
        duration: Option<JsUint>,
        origin: Option<Origin>,
        pointer_common_properties: PointerCommonProperties,
    ) -> Self {
        Self {
            pointer_move_action_type: "pointerMove".to_string(),
            x,
            y,
            duration,
            origin,
            pointer_common_properties,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WheelScrollAction {
    #[serde(rename = "type")]
    pub wheel_scroll_action_type: String,
    pub x: JsInt,
    pub y: JsInt,
    #[serde(rename = "deltaX")]
    pub delta_x: JsInt,
    #[serde(rename = "deltaY")]
    pub delta_y: JsInt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<JsUint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
}

impl WheelScrollAction {
    pub fn new(
        x: JsInt,
        y: JsInt,
        delta_x: JsInt,
        delta_y: JsInt,
        duration: Option<JsUint>,
        origin: Option<Origin>,
    ) -> Self {
        Self {
            wheel_scroll_action_type: "scroll".to_string(),
            x,
            y,
            delta_x,
            delta_y,
            duration,
            origin,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointerCommonProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<JsUint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<JsUint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pressure: Option<f64>,
    #[serde(rename = "tangentialPressure", skip_serializing_if = "Option::is_none")]
    pub tangential_pressure: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twist: Option<JsUint>,
    #[serde(rename = "altitudeAngle", skip_serializing_if = "Option::is_none")]
    pub altitude_angle: Option<f64>,
    #[serde(rename = "azimuthAngle", skip_serializing_if = "Option::is_none")]
    pub azimuth_angle: Option<f64>,
}

impl PointerCommonProperties {
    pub fn new(
        width: Option<JsUint>,
        height: Option<JsUint>,
        pressure: Option<f64>,
        tangential_pressure: Option<f64>,
        twist: Option<JsUint>,
        altitude_angle: Option<f64>,
        azimuth_angle: Option<f64>,
    ) -> Self {
        Self {
            width,
            height,
            pressure,
            tangential_pressure,
            twist,
            altitude_angle,
            azimuth_angle,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Origin {
    Viewport(String),
    Pointer(String),
    ElementOrigin(ElementOrigin),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseActions {
    pub method: String,
    pub params: ReleaseActionsParameters,
}

impl ReleaseActions {
    pub fn new(params: ReleaseActionsParameters) -> Self {
        Self {
            method: "input.releaseActions".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseActionsParameters {
    pub context: BrowsingContext,
}

impl ReleaseActionsParameters {
    pub fn new(context: BrowsingContext) -> Self {
        Self { context }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetFiles {
    pub method: String,
    pub params: SetFilesParameters,
}

impl SetFiles {
    pub fn new(params: SetFilesParameters) -> Self {
        Self {
            method: "input.setFiles".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetFilesParameters {
    pub context: BrowsingContext,
    pub element: SharedReference,
    pub files: Vec<String>,
}

impl SetFilesParameters {
    pub fn new(context: BrowsingContext, element: SharedReference, files: Vec<String>) -> Self {
        Self {
            context,
            element,
            files,
        }
    }
}
