use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// Represents the standard WebDriver BiDi events.
#[derive(Hash, Eq, PartialEq, Debug)]
pub enum EventType {
    BrowsingContextContextCreated,
    BrowsingContextContextDestroyed,
    BrowsingContextNavigationStarted,
    BrowsingContextFragmentNavigated,
    BrowsingContextHistoryUpdated,
    BrowsingContextDomContentLoaded,
    BrowsingContextLoad,
    BrowsingContextDownloadWillBegin,
    BrowsingContextNavigationAborted,
    BrowsingContextNavigationCommitted,
    BrowsingContextNavigationFailed,
    BrowsingContextUserPromptClosed,
    BrowsingContextUserPromptOpened,
    NetworkAuthRequired,
    NetworkBeforeRequestSent,
    NetworkFetchError,
    NetworkResponseCompleted,
    NetworkResponseStarted,
    ScriptMessage,
    ScriptRealmCreated,
    ScriptRealmDestroyed,
    LogEntryAdded,
}

/// Simple error type for parsing EventType.
#[derive(Debug)]
pub struct ParseEventTypeError;

impl fmt::Display for ParseEventTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid EventType string")
    }
}

impl Error for ParseEventTypeError {}

impl FromStr for EventType {
    type Err = ParseEventTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "browsingContext.contextCreated" => Ok(EventType::BrowsingContextContextCreated),
            "browsingContext.contextDestroyed" => Ok(EventType::BrowsingContextContextDestroyed),
            "browsingContext.navigationStarted" => Ok(EventType::BrowsingContextNavigationStarted),
            "browsingContext.fragmentNavigated" => Ok(EventType::BrowsingContextFragmentNavigated),
            "browsingContext.historyUpdated" => Ok(EventType::BrowsingContextHistoryUpdated),
            "browsingContext.domContentLoaded" => Ok(EventType::BrowsingContextDomContentLoaded),
            "browsingContext.load" => Ok(EventType::BrowsingContextLoad),
            "browsingContext.downloadWillBegin" => Ok(EventType::BrowsingContextDownloadWillBegin),
            "browsingContext.navigationAborted" => Ok(EventType::BrowsingContextNavigationAborted),
            "browsingContext.navigationCommitted" => {
                Ok(EventType::BrowsingContextNavigationCommitted)
            }
            "browsingContext.navigationFailed" => Ok(EventType::BrowsingContextNavigationFailed),
            "browsingContext.userPromptClosed" => Ok(EventType::BrowsingContextUserPromptClosed),
            "browsingContext.userPromptOpened" => Ok(EventType::BrowsingContextUserPromptOpened),
            "network.authRequired" => Ok(EventType::NetworkAuthRequired),
            "network.beforeRequestSent" => Ok(EventType::NetworkBeforeRequestSent),
            "network.fetchError" => Ok(EventType::NetworkFetchError),
            "network.responseCompleted" => Ok(EventType::NetworkResponseCompleted),
            "network.responseStarted" => Ok(EventType::NetworkResponseStarted),
            "script.message" => Ok(EventType::ScriptMessage),
            "script.realmCreated" => Ok(EventType::ScriptRealmCreated),
            "script.realmDestroyed" => Ok(EventType::ScriptRealmDestroyed),
            "log.entryAdded" => Ok(EventType::LogEntryAdded),
            _ => Err(ParseEventTypeError),
        }
    }
}
