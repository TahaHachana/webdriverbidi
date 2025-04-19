use serde::{Deserialize, Serialize};

use crate::model::browsing_context::BrowsingContextResult;
use crate::model::common::Extensible;
use crate::model::network::NetworkResult;
use crate::model::script::ScriptResult;
use crate::model::session::SessionResult;
use crate::model::storage::StorageResult;
use crate::model::web_extension::WebExtensionResult;

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
