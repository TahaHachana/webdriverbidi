use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type Extensible = HashMap<String, serde_json::Value>;

// -9007199254740991..9007199254740991
pub type JsInt = i64;
// 0..9007199254740991
pub type JsUint = u64;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EmptyParams {
    #[serde(flatten)]
    pub extensible: Extensible,
}

impl EmptyParams {
    pub fn new() -> Self {
        Self {
            extensible: Extensible::new(),
        }
    }
}

/// Deserialize a `js-uint`, reading a negative integer as `0`, for fields where
/// browsers send `-1` to mean "unknown".
pub(crate) fn uint_or_zero<'de, D: serde::Deserializer<'de>>(d: D) -> Result<JsUint, D::Error> {
    let value = <i128 as Deserialize>::deserialize(d)?;
    JsUint::try_from(value.max(0)).map_err(serde::de::Error::custom)
}

/// Deserializers for `type`-style discriminator fields.
///
/// The model represents many spec unions as `#[serde(untagged)]` enums whose
/// variants keep the discriminator as a plain `String`. Without checking its
/// value, serde picks the first variant whose fields fit, e.g. any remote value
/// would parse as `undefined`. These functions reject other values, so serde
/// falls through to the right variant.
pub(crate) mod tag {
    macro_rules! tags {
        ($($name:ident => $lit:literal),* $(,)?) => {$(
            pub(crate) fn $name<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
                let value = <String as serde::Deserialize>::deserialize(d)?;
                if value == $lit {
                    Ok(value)
                } else {
                    Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(&value),
                        &$lit,
                    ))
                }
            }
        )*};
    }

    tags! {
        array => "array",
        arraybuffer => "arraybuffer",
        audio_worklet => "audio-worklet",
        autodetect => "autodetect",
        base64 => "base64",
        bigint => "bigint",
        boolean => "boolean",
        canceled => "canceled",
        channel => "channel",
        complete => "complete",
        console => "console",
        date => "date",
        dedicated_worker => "dedicated-worker",
        direct => "direct",
        error => "error",
        function => "function",
        generator => "generator",
        htmlcollection => "htmlcollection",
        javascript => "javascript",
        manual => "manual",
        map => "map",
        node => "node",
        nodelist => "nodelist",
        null => "null",
        number => "number",
        object => "object",
        pac => "pac",
        paint_worklet => "paint-worklet",
        promise => "promise",
        proxy => "proxy",
        regexp => "regexp",
        service_worker => "service-worker",
        set => "set",
        shared_worker => "shared-worker",
        string => "string",
        symbol => "symbol",
        system => "system",
        typedarray => "typedarray",
        undefined => "undefined",
        weakmap => "weakmap",
        weakset => "weakset",
        window => "window",
        worker => "worker",
        worklet => "worklet",
    }
}
