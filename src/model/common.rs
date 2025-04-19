use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type Extensible = HashMap<String, serde_json::Value>;

// -9007199254740991..9007199254740991
pub type JsInt = i64;
// 0..9007199254740991
pub type JsUint = u64;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmptyParams {
    pub extensible: Extensible,
}

impl EmptyParams {
    pub fn new() -> Self {
        Self {
            extensible: Extensible::new(),
        }
    }
}
