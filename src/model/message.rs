use serde::{Deserialize, Serialize};

use crate::model::command::CommandResponse;
use crate::model::error::ErrorResponse;
use crate::model::event::Event;

#[allow(
    clippy::large_enum_variant,
    reason = "mirrors a spec union; boxing variants would make matching on it awkward for no measured gain"
)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Message {
    CommandResponse(CommandResponse),
    ErrorResponse(ErrorResponse),
    Event(Event),
}
