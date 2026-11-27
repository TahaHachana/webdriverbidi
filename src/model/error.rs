use serde::{Deserialize, Serialize};

use crate::model::common::{Extensible, JsUint};

/// An error response from the remote end (`ErrorResponse` in the spec).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    /// The id of the failed command; `None` if the remote end couldn't read one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsUint>,
    pub error: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stacktrace: Option<String>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

impl ErrorResponse {
    pub fn new(
        response_type: String,
        id: Option<JsUint>,
        error: ErrorCode,
        message: String,
        stacktrace: Option<String>,
        extensible: Extensible,
    ) -> Self {
        Self {
            response_type,
            id,
            error,
            message,
            stacktrace,
            extensible,
        }
    }
}

/// Defines `ErrorCode` with its spec strings, plus `Other` for codes this crate
/// doesn't know (browsers return codes from the spec's prose that its CDDL omits,
/// and new codes appear over time).
macro_rules! error_codes {
    ($($(#[$doc:meta])* $variant:ident => $code:literal,)*) => {
        /// A WebDriver BiDi error code.
        #[derive(Debug, PartialEq, Eq, Clone, Hash)]
        #[non_exhaustive]
        pub enum ErrorCode {
            $($(#[$doc])* $variant,)*
            /// A code this crate doesn't know, kept verbatim.
            Other(String),
        }

        impl ErrorCode {
            /// The code as it appears on the wire, e.g. `"no such frame"`.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $code,)*
                    Self::Other(code) => code,
                }
            }
        }

        impl From<&str> for ErrorCode {
            fn from(code: &str) -> Self {
                match code {
                    $($code => Self::$variant,)*
                    other => Self::Other(other.to_string()),
                }
            }
        }
    };
}

error_codes! {
    InvalidArgument => "invalid argument",
    InvalidSelector => "invalid selector",
    InvalidSessionId => "invalid session id",
    InvalidWebExtension => "invalid web extension",
    MoveTargetOutOfBounds => "move target out of bounds",

    /// Defined and used in the spec's prose (`browser.setClientWindowState`), but
    /// missing from its CDDL `ErrorCode` list.
    NoSuchClientWindow => "no such client window",
    NoSuchAlert => "no such alert",
    NoSuchNetworkCollector => "no such network collector",
    NoSuchElement => "no such element",
    NoSuchFrame => "no such frame",
    NoSuchHandle => "no such handle",
    NoSuchHistoryEntry => "no such history entry",
    NoSuchIntercept => "no such intercept",
    NoSuchNetworkData => "no such network data",
    NoSuchNode => "no such node",
    NoSuchRequest => "no such request",
    NoSuchScreencast => "no such screencast",
    NoSuchScript => "no such script",
    NoSuchStoragePartition => "no such storage partition",
    NoSuchUserContext => "no such user context",
    NoSuchWebExtension => "no such web extension",
    SessionNotCreated => "session not created",
    UnableToCaptureScreen => "unable to capture screen",
    UnableToCloseBrowser => "unable to close browser",
    UnableToSetCookie => "unable to set cookie",
    UnableToSetFileInput => "unable to set file input",
    UnavailableNetworkData => "unavailable network data",
    UnderspecifiedStoragePartition => "underspecified storage partition",
    UnknownCommand => "unknown command",
    UnknownError => "unknown error",
    UnsupportedOperation => "unsupported operation",
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ErrorCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let code = <std::borrow::Cow<'de, str> as Deserialize>::deserialize(deserializer)?;
        Ok(ErrorCode::from(code.as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn known_and_unknown_codes_round_trip() {
        let known: ErrorCode = serde_json::from_value(json!("no such frame")).unwrap();
        assert_eq!(known, ErrorCode::NoSuchFrame);
        assert_eq!(
            serde_json::to_value(&known).unwrap(),
            json!("no such frame")
        );

        let unknown: ErrorCode = serde_json::from_value(json!("vendor specific")).unwrap();
        assert_eq!(unknown, ErrorCode::Other("vendor specific".into()));
        assert_eq!(
            serde_json::to_value(&unknown).unwrap(),
            json!("vendor specific")
        );
        assert_eq!(unknown.to_string(), "vendor specific");
    }
}
