use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebExtensionCommand {
    Install(Install),
    Uninstall(Uninstall),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum WebExtensionResult {
    InstallResult(InstallResult),
}

pub type Extension = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct Install {
    pub method: String,
    pub params: InstallParameters,
}

impl Install {
    pub fn new(params: InstallParameters) -> Self {
        Self {
            method: "webExtension.install".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallParameters {
    #[serde(rename = "extensionData")]
    pub extension_data: ExtensionData,
}

impl InstallParameters {
    pub fn new(extension_data: ExtensionData) -> Self {
        Self { extension_data }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExtensionData {
    ExtensionArchivePath(ExtensionArchivePath),
    ExtensionBase64Encoded(ExtensionBase64Encoded),
    ExtensionPath(ExtensionPath),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionPath {
    #[serde(rename = "type")]
    pub extension_path_type: String,
    pub path: String,
}

impl ExtensionPath {
    pub fn new(path: String) -> Self {
        Self {
            extension_path_type: "path".to_string(),
            path,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionArchivePath {
    #[serde(rename = "type")]
    pub extension_archive_path_type: String,
    pub path: String,
}

impl ExtensionArchivePath {
    pub fn new(path: String) -> Self {
        Self {
            extension_archive_path_type: "archivePath".to_string(),
            path,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionBase64Encoded {
    #[serde(rename = "type")]
    pub extension_base64_encoded_type: String,
    pub value: String,
}

impl ExtensionBase64Encoded {
    pub fn new(value: String) -> Self {
        Self {
            extension_base64_encoded_type: "base64".to_string(),
            value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallResult {
    pub extension: Extension,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Uninstall {
    pub method: String,
    pub params: UninstallParameters,
}

impl Uninstall {
    pub fn new(params: UninstallParameters) -> Self {
        Self {
            method: "webExtension.uninstall".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UninstallParameters {
    pub extension: Extension,
}

impl UninstallParameters {
    pub fn new(extension: Extension) -> Self {
        Self { extension }
    }
}
