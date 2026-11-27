# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project follows [Semantic Versioning](https://semver.org/) (while below 1.0, a
minor version bump may contain breaking changes).

## [0.5.0] - 2026-11-27

Matches the WebDriver BiDi specification at
[w3c/webdriver-bidi@5931e50](https://github.com/w3c/webdriver-bidi/commit/5931e506984a03207b8c1e9829643cb9aa6b6b15),
as 0.4.0 does. Every command and event is now tested live against Chrome and Firefox,
and the bugs those tests found are fixed below.

### Added

- `WebDriverBiDiSession::set_message_observer`, with `MessageObserver` and
  `MessageDirection`: a callback that sees every message sent to and received from the
  remote end as raw JSON, e.g. for logging or recording traffic.
- Typed errors: `CommandError::error_response()` and `CommandError::error_code()` return
  the remote end's `ErrorResponse` / `ErrorCode`, and the error displays as
  ``Command returned error `<code>`: <message>``. `ErrorResponse` fields are public, and
  `ErrorCode` has `as_str()` and `Display`.
- `ErrorCode::Other(String)` keeps codes the crate doesn't know, so an unknown code no
  longer makes an `ErrorResponse` fail to parse.
- `CommandError::ConnectionClosed`, returned when the connection closes before a command is
  sent or answered.
- All model types implement `Clone` and `PartialEq`.
- `network::RequestData`, `network::FetchTimingInfo`, `script::StackFrame` and
  `session::StatusResult` keep fields outside the spec (vendor extensions such as Chrome's
  `goog:resourceType`) in a flattened `extensible` map instead of dropping them.

### Changed (breaking)

- `ErrorCode` has new variants, `NoSuchClientWindow` and `Other(String)`; it no longer
  derives `Serialize`/`Deserialize` but implements them (same JSON), and is
  `#[non_exhaustive]`.
- `CommandError` is `#[non_exhaustive]`; `OneshotReceiverError` is replaced by
  `ConnectionClosed`, and `WebSocketSendError` now holds a `Box<tungstenite::Error>`.
- The four types above have a new public `extensible` field, which breaks struct literals.
- `WebDriverBiDiSession::websocket_stream` is removed. The connection is now split into
  a write half, used to send commands, and a read half, owned by the task that receives
  messages; both are private. Use `send_command` to send raw commands and
  `set_message_observer` to see raw traffic.

### Fixed

- Every session's receive task polled the WebSocket in a loop that never waited,
  using a full CPU core for as long as the session existed and continuing after the
  connection closed. It now waits for messages and stops when the connection ends.
- A command waiting for its response when the connection closed waited for the
  60-second response timeout; it now fails immediately with `ConnectionClosed`.
- `ErrorCode` lacked `no such client window`, which the spec's prose defines (its CDDL list
  omits it) and Firefox returns for `browser.setClientWindowState` on an unknown window, so
  such error responses failed to parse as `ErrorResponse`.
- `input.performActions` sent `"parameters": null` for a pointer source without
  parameters; the field is now omitted, as the spec requires.
- `network.authRequired` events from Chrome failed to parse: Chrome 158 sends
  `response.status: -1`, which the spec doesn't allow (`js-uint`). A negative status is now
  read as `0`, Fetch's "no status" value; `ResponseData::status` stays a `u64`.

## [0.4.0] - 2026-11-27

### Changed (breaking)

- `ContextCreatedParameters::has_planned_navigation` is now `Option<bool>`, and `download` on
  `DownloadWillBeginParams`, `DownloadCanceledParams` and `DownloadCompleteParams` is now
  `Option<Download>`. Both fields are recent additions to the spec, and browsers don't send
  them yet; `None` means the browser didn't report the value.

### Fixed

- `browsingContext.contextCreated` events from Chrome 158 failed to parse, because Chrome
  doesn't send `hasPlannedNavigation` yet. Download events from browsers that don't send
  `download` failed the same way. 0.3.0 users should upgrade.

## [0.3.0] - 2026-11-27

Synced with the WebDriver BiDi specification at
[w3c/webdriver-bidi@5931e50](https://github.com/w3c/webdriver-bidi/commit/5931e506984a03207b8c1e9829643cb9aa6b6b15)
(2026-11-26).

### Added

- Commands:
  - `browser.setDownloadBehavior`
  - `browsingContext.setBypassCSP`, `browsingContext.startScreencast`,
    `browsingContext.stopScreencast`
  - `emulation.setForcedColorsModeThemeOverride`, `emulation.setMediaFeaturesOverride`,
    `emulation.setNetworkConditions`, `emulation.setScreenSettingsOverride`,
    `emulation.setScriptingEnabled`, `emulation.setScrollbarTypeOverride`,
    `emulation.setTextLayoutModeOverride`, `emulation.setTouchOverride`,
    `emulation.setUserAgentOverride`, `emulation.setViewportMetaOverride`
- `EventType::InputFileDialogOpened`, so handlers can be registered for
  `input.fileDialogOpened`.
- Fields from the spec:
  - an optional `user_context` on `BaseNavigationInfo`, `HistoryUpdatedParameters`,
    `UserPromptOpenedParameters`, `UserPromptClosedParameters`, `browsing_context::CreateResult`,
    `script::WindowRealmInfo`, `script::Source`, `input::FileDialogInfo` and
    `network::BaseParameters`;
  - `download` on `DownloadWillBeginParams`, `DownloadCanceledParams` and
    `DownloadCompleteParams`;
  - `image_size` on `CaptureScreenshotParameters`;
  - `has_planned_navigation` on `browsingContext.contextCreated` (see `ContextCreatedParameters`).
- `ErrorCode::NoSuchScreencast`.
- `input::FileDialogInfo` fields are now public.
- `EmptyParams` implements `Default`.
- Spec conformance tests (`tests/cddl_conformance.rs`) that validate every command,
  result, event and error code against the spec's CDDL, without a browser.
- Scripts: `scripts/update-cddl.sh` (regenerate `cddl/` from the spec),
  `scripts/publish.sh` (release checks and publishing) and
  `scripts/install-chromedriver.sh` (chromedriver matching the installed Chrome).
- `rust-version = "1.85"` is now declared.

### Changed (breaking)

- `CaptureScreenshotParameters::new` takes a fifth argument, `image_size: Option<ImageSize>`.
- `UnsubscribeByAttributesRequest` lost its `contexts` field (removed from the spec), so
  `UnsubscribeByAttributesRequest::new` takes only `events`.
- `script::EvaluateResult` is now `script::EvaluationResult`, matching the spec. Its variants
  are `EvaluationResultSuccess`, `EvaluationResultException` and `EmptyResult`, and
  `ScriptResult::EvaluateResult` is now `ScriptResult::EvaluationResult`. `EvaluateResult` and
  `CallFunctionResult` remain as type aliases, but code matching on the old variant names must
  be updated:

  ```rust
  // 0.2
  EvaluateResult::EvaluateResultSuccess(s) => { /* ... */ }
  // 0.3
  EvaluationResult::EvaluationResultSuccess(s) => { /* ... */ }
  ```
- `browsing_context::ContextCreated::params` is a `ContextCreatedParameters`
  (`info: Info` plus `has_planned_navigation`) instead of an `Info`.
- New public fields (listed under Added) break code that builds these structs with struct
  literals or destructures them exhaustively; use `..` in patterns.
- New enum variants break exhaustive `match`es on `BrowserCommand`, `BrowserResult`,
  `BrowsingContextCommand`, `BrowsingContextResult`, `EmulationCommand`, `NetworkCommand`,
  `NetworkResult`, `ScriptResult`, `ResultData`, `ErrorCode` and `EventType`.
- Variant structs of untagged enums (remote/local values, realm infos, proxy configurations,
  `network::BytesValue` members, download-end and log entries) now reject a mismatched `type`
  when deserialized. For example, deserializing `{"type": "string"}` into `UndefinedValue` is an
  error instead of silently succeeding.

### Deprecated

- `session::SubscriptionRequest`: renamed to `SubscribeParameters` in the spec.
- `session::Capabilities`: renamed to `CapabilityResponse` in the spec.
- `script::EvaluateResultSuccess` and `script::EvaluateResultException`: renamed to
  `EvaluationResultSuccess` and `EvaluationResultException`.

### Fixed

- `script.evaluate` and `script.callFunction` results lost their data for every non-primitive
  value (objects, arrays, nodes, maps, sets, functions, promises, windows, ...): they parsed as
  `undefined`, dropping `handle`, `value`, `sharedId` and `internalId`. Exception details were
  affected the same way.
- `network::BytesValue` was serialized as `{"StringValue": {...}}` instead of
  `{"type": "string", "value": ...}`, which broke request bodies, headers and cookies in
  `network.continueRequest`, `network.continueResponse`, `network.provideResponse`,
  `network.setExtraHeaders`, `storage.getCookies` and `storage.setCookie`. It also broke parsing
  of network events and cookie results.
- `session.new` results failed to deserialize (`sessionId` was read as `session_id`).
- Network events failed to deserialize (`isBlocked` and `redirectCount` were read in snake case).
- `browser.setClientWindowState` nested the window state under a `clientWindowNamedState` key
  instead of sending its fields directly in the parameters.
- `emulation.setScreenOrientationOverride` sent `null` for the orientation values.
- Snake-case keys sent where the spec has camelCase: `userText`
  (`browsingContext.handleUserPrompt`), `urlPatterns` (`network.addIntercept`) and
  `cacheBehavior` (`network.setCacheBehavior`).
- Storage commands sent an extra `"extensible": {}` key, and partition keys in storage results
  failed to deserialize.
- Commands without parameters (e.g. `session.status`, `browser.close`) sent
  `"params": {"extensible": {}}` instead of `"params": {}`.
- `log.entryAdded` console entries parsed as generic entries, losing `method` and `args`.
- Completed downloads (`browsingContext.downloadEnd`) parsed as canceled, losing `filepath`.
- Proxy configurations and realm infos could parse as the wrong variant.

[0.5.0]: https://github.com/TahaHachana/webdriverbidi/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/TahaHachana/webdriverbidi/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/TahaHachana/webdriverbidi/compare/62216c8...v0.3.0
