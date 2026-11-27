use serde::{Deserialize, Serialize};

use crate::model::browsing_context::BrowsingContextEvent;
use crate::model::common::Extensible;
use crate::model::input::InputEvent;
use crate::model::log::LogEvent;
use crate::model::network::NetworkEvent;
use crate::model::script::ScriptEvent;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    pub event_data: EventData,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[allow(
    clippy::large_enum_variant,
    reason = "mirrors a spec union; boxing variants would make matching on it awkward for no measured gain"
)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum EventData {
    BrowsingContextEvent(BrowsingContextEvent),
    InputEvent(InputEvent),
    LogEvent(LogEvent),
    NetworkEvent(NetworkEvent),
    ScriptEvent(ScriptEvent),
}
