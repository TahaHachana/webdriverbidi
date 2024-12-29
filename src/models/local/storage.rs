use crate::models::local::{network, Extensible};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum StorageResult {
    DeleteCookiesResult(DeleteCookiesResult),
    GetCookiesResult(GetCookiesResult),
    SetCookieResult(SetCookieResult),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PartitionKey {
    #[serde(rename = "userContext", skip_serializing_if = "Option::is_none")]
    user_context: Option<String>,
    #[serde(rename = "sourceOrigin", skip_serializing_if = "Option::is_none")]
    source_origin: Option<String>,
    #[serde(flatten)]
    extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCookiesResult {
    cookies: Vec<network::Cookie>,
    #[serde(rename = "partitionKey")]
    partition_key: PartitionKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetCookieResult {
    #[serde(rename = "partitionKey")]
    partition_key: PartitionKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteCookiesResult {
    #[serde(rename = "partitionKey")]
    partition_key: PartitionKey,
}