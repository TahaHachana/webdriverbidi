use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum WebExtensionResult {
    InstallResult(InstallResult),
}

pub type Extension = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallResult {
    pub extension: Extension,
}
