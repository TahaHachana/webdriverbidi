use serde::{Deserialize, Serialize};

use crate::model::browser::BrowserResult;
use crate::model::browsing_context::BrowsingContextResult;
use crate::model::common::Extensible;
use crate::model::network::NetworkResult;
use crate::model::script::ScriptResult;
use crate::model::session::SessionResult;
use crate::model::storage::StorageResult;
use crate::model::web_extension::WebExtensionResult;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum ResultData {
    BrowserResult(BrowserResult),
    BrowsingContextResult(BrowsingContextResult),
    NetworkResult(NetworkResult),
    ScriptResult(ScriptResult),
    SessionResult(SessionResult),
    StorageResult(StorageResult),
    WebExtensionResult(WebExtensionResult),
    /// Fallback for commands whose result is `EmptyResult` (most emulation and
    /// input commands, among others). Must stay last: untagged enums try
    /// variants in order and this matches any object.
    EmptyResult(EmptyResult),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct EmptyResult {
    #[serde(flatten)]
    pub extensible: Extensible,
}
