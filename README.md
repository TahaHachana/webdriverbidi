# webdriverbidi

## Overview

The webdriverbidi library provides an interface for interacting with web browsers through the WebDriver BiDi (Bidirectional) protocol. This library allows you to create and manage WebDriver sessions, send commands, and handle responses asynchronously through WebSockets.

## Features

- Create and manage WebDriver BiDi sessions
- Send commands
- Handle events asynchronously

## Getting Started

### Prerequisites

- Rust and Cargo installed
- A WebDriver server that supports the BiDi protocol

### Installation

Add the following to your `Cargo.toml` (the example below will also require tokio with full features):

```toml
[dependencies]
webdriverbidi = "0.4.0"
```

### Usage

Start a WebDriver BiDi compliant server. To get a chromedriver that matches your installed Chrome, run
`scripts/install-chromedriver.sh` (installs into the git-ignored `drivers/` directory; see `--help` for
`--channel`, `--version` and `--dir`).

```bash
$ geckodriver --host=localhost --port=4444
# chromedriver --host=localhost --port=4444
# ./msedgedriver --host=localhost --port=4444
```

Create a new Rust project and add the following code to `src/main.rs`:

```rust
use anyhow::Result;
use tokio::time;

use webdriverbidi::model::browsing_context::{
    GetTreeParameters, NavigateParameters, ReadinessState,
};
use webdriverbidi::session::WebDriverBiDiSession;
use webdriverbidi::webdriver::capabilities::CapabilitiesRequest;

const HOST: &str = "localhost";
const PORT: u16 = 4444;

async fn sleep_for_secs(secs: u64) {
    time::sleep(time::Duration::from_secs(secs)).await
}

/// Initialize a new WebDriver BiDi session.
pub async fn init_session() -> Result<WebDriverBiDiSession> {
    let capabilities = CapabilitiesRequest::default();
    let mut session = WebDriverBiDiSession::new(HOST.into(), PORT, capabilities);
    session.start().await?;
    Ok(session)
}

/// Retrieve the browsing context at the specified index.
pub async fn get_context(session: &mut WebDriverBiDiSession, idx: usize) -> Result<String> {
    let get_tree_params = GetTreeParameters::new(None, None);
    let get_tree_rslt = session.browsing_context_get_tree(get_tree_params).await?;
    if let Some(context_entry) = get_tree_rslt.contexts.get(idx) {
        Ok(context_entry.context.clone())
    } else {
        anyhow::bail!(
            "No browsing context found at index {}. Available contexts: {}",
            idx,
            get_tree_rslt.contexts.len()
        );
    }
}

/// Navigate to the specified URL and wait for the document to completely load.
pub async fn navigate(session: &mut WebDriverBiDiSession, ctx: String, url: String) -> Result<()> {
    let navigate_params = NavigateParameters::new(ctx, url, Some(ReadinessState::Complete));
    session.browsing_context_navigate(navigate_params).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut session = init_session().await?;
    let ctx = get_context(&mut session, 0).await?;

    let url = String::from("https://www.rust-lang.org/");
    navigate(&mut session, ctx, url).await?;

    sleep_for_secs(1).await;
    session.close().await?;
    Ok(())
}
```

## Module Coverage

### session
#### Types
- [x] session.CapabilitiesRequest
- [x] session.CapabilityRequest
- [x] session.CapabilityResponse
- [x] session.ProxyConfiguration
- [x] session.UserPromptHandler
- [x] session.UserPromptHandlerType
- [x] session.Subscription
- [x] session.SubscribeParameters
- [x] session.UnsubscribeByIDRequest
- [x] session.UnsubscribeByAttributesRequest

#### Commands
- [x] session.status
- [x] session.new
- [x] session.end
- [x] session.subscribe
- [x] session.unsubscribe

### browser
#### Types
- [x] browser.ClientWindow
- [x] browser.ClientWindowInfo
- [x] browser.DownloadBehavior
- [x] browser.UserContext
- [x] browser.UserContextInfo

#### Commands
- [x] browser.close
- [x] browser.createUserContext
- [x] browser.getClientWindows
- [x] browser.getUserContexts
- [x] browser.removeUserContext
- [x] browser.setClientWindowState
- [x] browser.setDownloadBehavior

### browsingContext
#### Types
- [x] browsingContext.BrowsingContext
- [x] browsingContext.BaseInfo
- [x] browsingContext.Download
- [x] browsingContext.Info
- [x] browsingContext.Locator
- [x] browsingContext.Navigation
- [x] browsingContext.NavigationInfo
- [x] browsingContext.ReadinessState
- [x] browsingContext.Screencast
- [x] browsingContext.UserPromptType

#### Commands
- [x] browsingContext.activate
- [x] browsingContext.captureScreenshot
- [x] browsingContext.close
- [x] browsingContext.create
- [x] browsingContext.getTree
- [x] browsingContext.handleUserPrompt
- [x] browsingContext.locateNodes
- [x] browsingContext.navigate
- [x] browsingContext.print
- [x] browsingContext.reload
- [x] browsingContext.setBypassCSP
- [x] browsingContext.setViewport
- [x] browsingContext.startScreencast
- [x] browsingContext.stopScreencast
- [x] browsingContext.traverseHistory

#### Events
- [x] browsingContext.contextCreated
- [x] browsingContext.contextDestroyed
- [x] browsingContext.navigationStarted
- [x] browsingContext.fragmentNavigated
- [x] browsingContext.historyUpdated
- [x] browsingContext.domContentLoaded
- [x] browsingContext.load
- [x] browsingContext.downloadWillBegin
- [x] browsingContext.downloadEnd
- [x] browsingContext.navigationAborted
- [x] browsingContext.navigationCommitted
- [x] browsingContext.navigationFailed
- [x] browsingContext.userPromptClosed
- [x] browsingContext.userPromptOpened

### emulation
#### Commands
- [x] emulation.setForcedColorsModeThemeOverride
- [x] emulation.setGeolocationOverride
- [x] emulation.setLocaleOverride
- [x] emulation.setMediaFeaturesOverride
- [x] emulation.setNetworkConditions
- [x] emulation.setScreenOrientationOverride
- [x] emulation.setScreenSettingsOverride
- [x] emulation.setScriptingEnabled
- [x] emulation.setScrollbarTypeOverride
- [x] emulation.setTextLayoutModeOverride
- [x] emulation.setTimezoneOverride
- [x] emulation.setTouchOverride
- [x] emulation.setUserAgentOverride
- [x] emulation.setViewportMetaOverride

### network
#### Types
- [x] network.AuthChallenge
- [x] network.AuthCredentials
- [x] network.BaseParameters
- [x] network.BytesValue
- [x] network.Collector
- [x] network.CollectorType
- [x] network.Cookie
- [x] network.CookieHeader
- [x] network.DataType
- [x] network.FetchTimingInfo
- [x] network.Header
- [x] network.Initiator
- [x] network.Intercept
- [x] network.Request
- [x] network.RequestData
- [x] network.ResponseContent
- [x] network.ResponseData
- [x] network.SetCookieHeader
- [x] network.UrlPattern

#### Commands
- [x] network.addDataCollector
- [x] network.addIntercept
- [x] network.continueRequest
- [x] network.continueResponse
- [x] network.continueWithAuth
- [x] network.disownData
- [x] network.failRequest
- [x] network.getData
- [x] network.provideResponse
- [x] network.removeDataCollector
- [x] network.removeIntercept
- [x] network.setCacheBehavior
- [x] network.setExtraHeaders

#### Events
- [x] network.authRequired
- [x] network.beforeRequestSent
- [x] network.fetchError
- [x] network.responseCompleted
- [x] network.responseStarted

### script
#### Types
- [x] script.Channel
- [x] script.ChannelValue
- [x] script.EvaluationResult
- [x] script.ExceptionDetails
- [x] script.Handle
- [x] script.InternalId
- [x] script.LocalValue
- [x] script.PreloadScript
- [x] script.Realm
- [x] script.PrimitiveProtocolValue
- [x] script.RealmInfo
- [x] script.RealmType
- [x] script.RemoteReference
- [x] script.RemoteValue
- [x] script.ResultOwnership
- [x] script.SerializationOptions
- [x] script.SharedId
- [x] script.StackFrame
- [x] script.StackTrace
- [x] script.Source
- [x] script.Target

#### Commands
- [x] script.addPreloadScript
- [x] script.disown
- [x] script.callFunction
- [x] script.evaluate
- [x] script.getRealms
- [x] script.removePreloadScript

#### Events
- [x] script.message
- [x] script.realmCreated
- [x] script.realmDestroyed

### storage
#### Types
- [x] storage.PartitionKey

#### Commands
- [x] storage.getCookies
- [x] storage.setCookie
- [x] storage.deleteCookies

### log
#### Types
- [x] log.LogEntry

#### Events
- [x] log.entryAdded

### input
#### Types
- [x] input.ElementOrigin

#### Commands
- [x] input.performActions
- [x] input.releaseActions
- [x] input.setFiles

#### Events
- [x] input.fileDialogOpened

### webExtension
#### Types
- [x] webExtension.Extension

#### Commands
- [x] webExtension.install
- [x] webExtension.uninstall

## Updating the CDDL

The `cddl/` directory holds the CDDL definitions extracted from the
[WebDriver BiDi spec](https://github.com/w3c/webdriver-bidi). To refresh them (requires `git` and `python3`):

```bash
scripts/update-cddl.sh          # latest main
scripts/update-cddl.sh <ref>    # a specific branch or tag
```

Then review `git diff cddl/` and update `src/model`, `src/commands`, `src/session.rs` and this README to match.

## Testing

```bash
cargo test --lib --test cddl_conformance   # no browser needed; runs in CI
```

`tests/cddl_conformance.rs` checks the crate against the spec's CDDL:

- every command, built through the public API, is validated against `cddl/remote.cddl`;
- spec-valid JSON for every command result and event (validated against `cddl/local.cddl`)
  must deserialize into the crate's types and serialize back without losing fields;
- coverage tests fail if the spec has a command, result, event or error code the suite
  doesn't exercise, so `scripts/update-cddl.sh` followed by `cargo test` lists what to implement.

The other files in `tests/` (one per spec module) are live integration tests against real
browsers. They cover every command and event the crate implements, and run in CI on Chrome and
Firefox:

```bash
scripts/install-chromedriver.sh && drivers/chromedriver --port=4444 &
cargo test --tests -- --test-threads=1

scripts/install-geckodriver.sh && drivers/geckodriver --port=4444 &
WEBDRIVERBIDI_BROWSER=firefox cargo test --tests -- --test-threads=1
```

Every message a test sends or receives is also checked against the spec: commands must match
`cddl/remote.cddl`, and everything received must parse into the crate's types without losing
data. Messages where the browser itself deviates from the spec are printed as
`SPEC DEVIATION (browser)` (run with `-- --nocapture` to see them), and fail the test only with
`WEBDRIVERBIDI_STRICT_SPEC=1`.

The tests run headless; `WEBDRIVERBIDI_HEADED=1`, `WEBDRIVERBIDI_HOST` and `WEBDRIVERBIDI_PORT`
change that. See `tests/support/` for the harness (`with_session`, `inline` test pages,
`subscribe` for events, and `supported`, which skips commands a browser doesn't implement yet).

## Releasing

```bash
scripts/publish.sh --dry-run    # run every check and `cargo publish --dry-run`
scripts/publish.sh              # publish to crates.io, then tag and push vX.Y.Z
```

Bump `version` in `Cargo.toml` and the version in the installation snippet above, and add a section for the release to [CHANGELOG.md](CHANGELOG.md), first. The script checks all three. You must be logged in to crates.io (`cargo login`).

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
