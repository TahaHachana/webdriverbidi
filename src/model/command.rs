use serde::{Deserialize, Serialize};

use crate::model::browser::BrowserCommand;
use crate::model::browsing_context::BrowsingContextCommand;
use crate::model::common::{Extensible, JsUint};
use crate::model::emulation::EmulationCommand;
use crate::model::input::InputCommand;
use crate::model::network::NetworkCommand;
use crate::model::result::ResultData;
use crate::model::script::ScriptCommand;
use crate::model::session::SessionCommand;
use crate::model::storage::StorageCommand;
use crate::model::web_extension::WebExtensionCommand;

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
    EmulationCommand(EmulationCommand),
    InputCommand(InputCommand),
    NetworkCommand(NetworkCommand),
    ScriptCommand(ScriptCommand),
    SessionCommand(SessionCommand),
    StorageCommand(StorageCommand),
    WebExtensionCommand(WebExtensionCommand),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResponse {
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
