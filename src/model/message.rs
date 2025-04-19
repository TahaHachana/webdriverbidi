use serde::{Deserialize, Serialize};

use crate::model::command::CommandResponse;
use crate::model::error::ErrorResponse;
use crate::model::event::Event;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    CommandResponse(CommandResponse),
    ErrorResponse(ErrorResponse),
    Event(Event),
}
