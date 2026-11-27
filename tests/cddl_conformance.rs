//! Spec conformance tests that need no browser.
//!
//! * Every command is built through the crate's public API, serialized exactly
//!   as it goes on the wire, and validated against its rule in
//!   `cddl/remote.cddl` (what a browser accepts).
//! * Every command result and event is given as spec-valid JSON. Each fixture is
//!   first validated against `cddl/local.cddl` (what a browser sends), then
//!   deserialized into the crate's types and serialized back. The round trip
//!   must not drop or change any field.
//! * Coverage tests fail when the spec has a command, result, event or error
//!   code that these tests don't exercise. After running
//!   `scripts/update-cddl.sh`, they list what is new.
//!
//! Run with: `cargo test --test cddl_conformance`

mod support;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use webdriverbidi::model::browser as b;
use webdriverbidi::model::browsing_context as bc;
use webdriverbidi::model::common::{EmptyParams, Extensible};
use webdriverbidi::model::emulation as em;
use webdriverbidi::model::error::{ErrorCode, ErrorResponse};
use webdriverbidi::model::input as inp;
use webdriverbidi::model::log as lg;
use webdriverbidi::model::network as net;
use webdriverbidi::model::result::EmptyResult;
use webdriverbidi::model::script as sc;
use webdriverbidi::model::session as ses;
use webdriverbidi::model::storage as st;
use webdriverbidi::model::web_extension as wx;

use support::cddl::*;

// ---------------------------------------------------------------------------
// Builders for commonly used values
// ---------------------------------------------------------------------------

fn s(v: &str) -> String {
    v.to_string()
}

fn ctxs() -> Option<Vec<String>> {
    Some(vec![s("ctx-1"), s("ctx-2")])
}

fn ucs() -> Option<Vec<String>> {
    Some(vec![s("user-ctx-1")])
}

fn shared_ref(id: &str) -> sc::SharedReference {
    sc::SharedReference {
        shared_id: s(id),
        handle: Some(s("handle-1")),
        extensible: Extensible::new(),
    }
}

fn string_bytes(v: &str) -> net::BytesValue {
    net::BytesValue::StringValue(net::StringValue {
        value_type: s("string"),
        value: s(v),
    })
}

fn base64_bytes(v: &str) -> net::BytesValue {
    net::BytesValue::Base64Value(net::Base64Value {
        value_type: s("base64"),
        value: s(v),
    })
}

fn header(name: &str) -> net::Header {
    net::Header::new(s(name), string_bytes("value"))
}

fn set_cookie_header() -> net::SetCookieHeader {
    net::SetCookieHeader::new(
        s("session"),
        string_bytes("abc"),
        Some(s("example.com")),
        Some(true),
        Some(s("Wed, 21 Oct 2026 07:28:00 GMT")),
        Some(3600),
        Some(s("/")),
        Some(net::SameSite::Lax),
        Some(true),
    )
}

fn serialization_options() -> sc::SerializationOptions {
    sc::SerializationOptions {
        max_dom_depth: Some(1),
        max_object_depth: Some(2),
        include_shadow_tree: Some(sc::IncludeShadowTree::Open),
    }
}

fn channel_value() -> sc::ChannelValue {
    sc::ChannelValue {
        value_type: s("channel"),
        value: sc::ChannelProperties {
            channel: s("channel-1"),
            serialization_options: Some(serialization_options()),
            ownership: Some(sc::ResultOwnership::Root),
        },
    }
}

fn primitive(p: sc::PrimitiveProtocolValue) -> sc::LocalValue {
    sc::LocalValue::PrimitiveProtocolValue(p)
}

fn string_local(v: &str) -> sc::LocalValue {
    primitive(sc::PrimitiveProtocolValue::StringValue(
        sc::StringValue::new(s(v)),
    ))
}

fn number_local(v: sc::NumberOrSpecialNumber) -> sc::LocalValue {
    primitive(sc::PrimitiveProtocolValue::NumberValue(sc::NumberValue {
        value_type: s("number"),
        value: v,
    }))
}

/// One `script.LocalValue` of every kind.
fn local_values() -> Vec<sc::LocalValue> {
    use sc::PrimitiveProtocolValue as P;
    vec![
        sc::LocalValue::RemoteReference(sc::RemoteReference::SharedReference(shared_ref("node-1"))),
        sc::LocalValue::RemoteReference(sc::RemoteReference::RemoteObjectReference(
            sc::RemoteObjectReference {
                handle: s("handle-2"),
                shared_id: Some(s("node-2")),
                extensible: Extensible::new(),
            },
        )),
        primitive(P::UndefinedValue(sc::UndefinedValue {
            value_type: s("undefined"),
        })),
        primitive(P::NullValue(sc::NullValue {
            value_type: s("null"),
        })),
        string_local("text"),
        number_local(sc::NumberOrSpecialNumber::Number(1.5)),
        number_local(sc::NumberOrSpecialNumber::SpecialNumber(
            sc::SpecialNumber::NaN,
        )),
        number_local(sc::NumberOrSpecialNumber::SpecialNumber(
            sc::SpecialNumber::NegativeZero,
        )),
        primitive(P::BooleanValue(sc::BooleanValue {
            value_type: s("boolean"),
            value: true,
        })),
        primitive(P::BigIntValue(sc::BigIntValue {
            value_type: s("bigint"),
            value: s("123"),
        })),
        sc::LocalValue::ChannelValue(channel_value()),
        sc::LocalValue::ArrayLocalValue(sc::ArrayLocalValue {
            value_type: s("array"),
            value: vec![string_local("a")],
        }),
        sc::LocalValue::DateLocalValue(sc::DateLocalValue {
            value_type: s("date"),
            value: s("2026-11-27T00:00:00.000Z"),
        }),
        sc::LocalValue::MapLocalValue(sc::MapLocalValue {
            value_type: s("map"),
            value: sc::MappingLocalValue(vec![
                (sc::LocalValueOrText::Text(s("key")), string_local("v")),
                (
                    sc::LocalValueOrText::LocalValue(string_local("k")),
                    string_local("v"),
                ),
            ]),
        }),
        sc::LocalValue::ObjectLocalValue(sc::ObjectLocalValue {
            value_type: s("object"),
            value: sc::MappingLocalValue(vec![(
                sc::LocalValueOrText::Text(s("key")),
                string_local("v"),
            )]),
        }),
        sc::LocalValue::RegExpLocalValue(sc::RegExpLocalValue {
            value_type: s("regexp"),
            value: sc::RegExpValue {
                pattern: s("a+"),
                flags: Some(s("gi")),
            },
        }),
        sc::LocalValue::SetLocalValue(sc::SetLocalValue {
            value_type: s("set"),
            value: vec![string_local("a")],
        }),
    ]
}

fn pointer_props() -> inp::PointerCommonProperties {
    inp::PointerCommonProperties::new(
        Some(2),
        Some(3),
        Some(0.5),
        Some(-0.5),
        Some(45),
        Some(1.0),
        Some(2.0),
    )
}

fn capability_request(browser: &str) -> ses::CapabilityRequest {
    ses::CapabilityRequest {
        accept_insecure_certs: Some(true),
        browser_name: Some(s(browser)),
        browser_version: Some(s("158")),
        platform_name: Some(s("linux")),
        proxy: Some(manual_proxy()),
        unhandled_prompt_behavior: Some(prompt_handler()),
        extensible: Extensible::new(),
    }
}

fn manual_proxy() -> ses::ProxyConfiguration {
    ses::ProxyConfiguration::ManualProxyConfiguration(ses::ManualProxyConfiguration {
        proxy_type: s("manual"),
        http_proxy: Some(s("proxy:8080")),
        ssl_proxy: Some(s("proxy:8443")),
        socks_proxy: Some(ses::SocksProxyConfiguration {
            socks_proxy: s("socks:1080"),
            socks_version: 5,
        }),
        no_proxy: Some(vec![s("localhost")]),
        extensible: Extensible::new(),
    })
}

fn prompt_handler() -> ses::UserPromptHandler {
    use ses::UserPromptHandlerType as H;
    ses::UserPromptHandler {
        alert: Some(H::Accept),
        before_unload: Some(H::Dismiss),
        confirm: Some(H::Ignore),
        default: Some(H::Accept),
        file: Some(H::Dismiss),
        prompt: Some(H::Ignore),
    }
}

fn all_media_features() -> em::MediaFeatures {
    em::MediaFeatures {
        any_hover: Some(Some(em::Hover::Hover)),
        any_pointer: Some(Some(em::Pointer::Coarse)),
        color: Some(Some(8)),
        color_gamut: Some(Some(em::ColorGamut::P3)),
        color_index: Some(Some(0)),
        display_mode: Some(Some(em::DisplayMode::PictureInPicture)),
        dynamic_range: Some(Some(em::DynamicRange::High)),
        environment_blending: Some(Some(em::EnvironmentBlending::Additive)),
        forced_colors: Some(Some(em::ForcedColors::Active)),
        grid: Some(Some(0)),
        horizontal_viewport_segments: Some(Some(1)),
        hover: Some(Some(em::Hover::None)),
        inverted_colors: Some(Some(em::InvertedColors::Inverted)),
        monochrome: Some(Some(0)),
        nav_controls: Some(Some(em::NavControls::Back)),
        overflow_block: Some(Some(em::OverflowBlock::OptionalPaged)),
        overflow_inline: Some(Some(em::OverflowInline::Scroll)),
        pointer: Some(Some(em::Pointer::Fine)),
        prefers_color_scheme: Some(Some(em::PrefersColorScheme::Dark)),
        prefers_contrast: Some(Some(em::PrefersContrast::NoPreference)),
        prefers_reduced_data: Some(Some(em::PrefersReduced::Reduce)),
        prefers_reduced_motion: Some(Some(em::PrefersReduced::NoPreference)),
        prefers_reduced_transparency: Some(None),
        scan: Some(Some(em::Scan::Progressive)),
        scripting: Some(Some(em::Scripting::InitialOnly)),
        update: Some(Some(em::Update::Slow)),
        vertical_viewport_segments: Some(Some(2)),
        video_color_gamut: Some(Some(em::ColorGamut::Rec2020)),
        video_dynamic_range: Some(Some(em::DynamicRange::Standard)),
    }
}

// ---------------------------------------------------------------------------
// Commands (remote end input)
// ---------------------------------------------------------------------------

struct CommandCase {
    label: &'static str,
    json: Value,
}

/// Serialize a command exactly as `define_command!` puts it on the wire:
/// `{ "id": <n>, "method": ..., "params": ... }`.
fn cmd(label: &'static str, command: impl Serialize) -> CommandCase {
    let mut json = serde_json::to_value(command).expect("serialize command");
    json.as_object_mut()
        .expect("command serializes to an object")
        .insert(s("id"), json!(1));
    CommandCase { label, json }
}

fn command_cases() -> Vec<CommandCase> {
    vec![
        // browser
        cmd("browser.close", b::Close::new(EmptyParams::new())),
        cmd(
            "browser.createUserContext (manual proxy)",
            b::CreateUserContext::new(b::CreateUserContextParameters::new(
                Some(true),
                Some(manual_proxy()),
                Some(prompt_handler()),
            )),
        ),
        cmd(
            "browser.createUserContext (autodetect proxy)",
            b::CreateUserContext::new(b::CreateUserContextParameters::new(
                None,
                Some(ses::ProxyConfiguration::AutodetectProxyConfiguration(
                    ses::AutodetectProxyConfiguration {
                        proxy_type: s("autodetect"),
                        extensible: Extensible::new(),
                    },
                )),
                None,
            )),
        ),
        cmd(
            "browser.createUserContext (direct proxy)",
            b::CreateUserContext::new(b::CreateUserContextParameters::new(
                None,
                Some(ses::ProxyConfiguration::DirectProxyConfiguration(
                    ses::DirectProxyConfiguration {
                        proxy_type: s("direct"),
                        extensible: Extensible::new(),
                    },
                )),
                None,
            )),
        ),
        cmd(
            "browser.createUserContext (pac proxy)",
            b::CreateUserContext::new(b::CreateUserContextParameters::new(
                None,
                Some(ses::ProxyConfiguration::PacProxyConfiguration(
                    ses::PacProxyConfiguration {
                        proxy_type: s("pac"),
                        proxy_autoconfig_url: s("http://example.com/proxy.pac"),
                        extensible: Extensible::new(),
                    },
                )),
                None,
            )),
        ),
        cmd(
            "browser.createUserContext (system proxy)",
            b::CreateUserContext::new(b::CreateUserContextParameters::new(
                None,
                Some(ses::ProxyConfiguration::SystemProxyConfiguration(
                    ses::SystemProxyConfiguration {
                        proxy_type: s("system"),
                        extensible: Extensible::new(),
                    },
                )),
                None,
            )),
        ),
        cmd(
            "browser.getClientWindows",
            b::GetClientWindows::new(EmptyParams::new()),
        ),
        cmd(
            "browser.getUserContexts",
            b::GetUserContexts::new(EmptyParams::new()),
        ),
        cmd(
            "browser.removeUserContext",
            b::RemoveUserContext::new(b::RemoveUserContextParameters::new(s("user-ctx-1"))),
        ),
        cmd(
            "browser.setClientWindowState (named)",
            b::SetClientWindowState::new(b::SetClientWindowStateParameters::new(
                s("window-1"),
                b::ClientWindowNamedOrRectState::ClientWindowNamedState(
                    b::ClientWindowNamedState::new(b::ClientWindowState::Minimized),
                ),
            )),
        ),
        cmd(
            "browser.setClientWindowState (rect)",
            b::SetClientWindowState::new(b::SetClientWindowStateParameters::new(
                s("window-1"),
                b::ClientWindowNamedOrRectState::ClientWindowRectState(
                    b::ClientWindowRectState::new(
                        s("normal"),
                        Some(800),
                        Some(600),
                        Some(-10),
                        Some(20),
                    ),
                ),
            )),
        ),
        cmd(
            "browser.setDownloadBehavior (allowed)",
            b::SetDownloadBehavior::new(b::SetDownloadBehaviorParameters::new(
                Some(b::DownloadBehavior::Allowed {
                    destination_folder: s("/tmp/downloads"),
                }),
                ucs(),
            )),
        ),
        cmd(
            "browser.setDownloadBehavior (denied)",
            b::SetDownloadBehavior::new(b::SetDownloadBehaviorParameters::new(
                Some(b::DownloadBehavior::Denied),
                None,
            )),
        ),
        cmd(
            "browser.setDownloadBehavior (reset)",
            b::SetDownloadBehavior::new(b::SetDownloadBehaviorParameters::new(None, None)),
        ),
        // browsingContext
        cmd(
            "browsingContext.activate",
            bc::Activate::new(bc::ActivateParameters::new(s("ctx-1"))),
        ),
        cmd(
            "browsingContext.captureScreenshot (box clip)",
            bc::CaptureScreenshot::new(bc::CaptureScreenshotParameters::new(
                s("ctx-1"),
                Some(bc::CaptureScreenshotParametersOrigin::Document),
                Some(bc::ImageFormat::new(s("image/jpeg"), Some(0.5))),
                Some(bc::ClipRectangle::BoxClipRectangle(
                    bc::BoxClipRectangle::new(1.0, 2.0, 30.0, 40.0),
                )),
                Some(bc::ImageSize::new(Some(100), Some(200))),
            )),
        ),
        cmd(
            "browsingContext.captureScreenshot (element clip)",
            bc::CaptureScreenshot::new(bc::CaptureScreenshotParameters::new(
                s("ctx-1"),
                Some(bc::CaptureScreenshotParametersOrigin::Viewport),
                None,
                Some(bc::ClipRectangle::ElementClipRectangle(
                    bc::ElementClipRectangle::new(shared_ref("node-1")),
                )),
                None,
            )),
        ),
        cmd(
            "browsingContext.close",
            bc::Close::new(bc::CloseParameters::new(s("ctx-1"), Some(true))),
        ),
        cmd(
            "browsingContext.create (tab)",
            bc::Create::new(bc::CreateParameters::new(
                bc::CreateType::Tab,
                Some(s("ctx-1")),
                Some(true),
                Some(s("user-ctx-1")),
            )),
        ),
        cmd(
            "browsingContext.create (window)",
            bc::Create::new(bc::CreateParameters::new(
                bc::CreateType::Window,
                None,
                None,
                None,
            )),
        ),
        cmd(
            "browsingContext.getTree",
            bc::GetTree::new(bc::GetTreeParameters::new(Some(2), Some(s("ctx-1")))),
        ),
        cmd(
            "browsingContext.handleUserPrompt",
            bc::HandleUserPrompt::new(bc::HandleUserPromptParameters::new(
                s("ctx-1"),
                Some(true),
                Some(s("answer")),
            )),
        ),
        cmd(
            "browsingContext.locateNodes (accessibility)",
            bc::LocateNodes::new(bc::LocateNodesParameters::new(
                s("ctx-1"),
                bc::Locator::AccessibilityLocator(bc::AccessibilityLocator::new(
                    bc::AccessibilityLocatorValue::new(Some(s("Submit")), Some(s("button"))),
                )),
                Some(5),
                Some(serialization_options()),
                Some(vec![shared_ref("node-1")]),
            )),
        ),
        cmd(
            "browsingContext.locateNodes (css)",
            bc::LocateNodes::new(bc::LocateNodesParameters::new(
                s("ctx-1"),
                bc::Locator::CssLocator(bc::CssLocator::new(s("div > p"))),
                None,
                None,
                None,
            )),
        ),
        cmd(
            "browsingContext.locateNodes (context)",
            bc::LocateNodes::new(bc::LocateNodesParameters::new(
                s("ctx-1"),
                bc::Locator::ContextLocator(bc::ContextLocator::new(bc::ContextValue::new(s(
                    "ctx-2",
                )))),
                None,
                None,
                None,
            )),
        ),
        cmd(
            "browsingContext.locateNodes (innerText)",
            bc::LocateNodes::new(bc::LocateNodesParameters::new(
                s("ctx-1"),
                bc::Locator::InnerTextLocator(bc::InnerTextLocator::new(
                    s("hello"),
                    Some(true),
                    Some(bc::InnerTextLocatorMatchType::Partial),
                    Some(3),
                )),
                None,
                None,
                None,
            )),
        ),
        cmd(
            "browsingContext.locateNodes (xpath)",
            bc::LocateNodes::new(bc::LocateNodesParameters::new(
                s("ctx-1"),
                bc::Locator::XPathLocator(bc::XPathLocator::new(s("//p"))),
                None,
                None,
                None,
            )),
        ),
        cmd(
            "browsingContext.navigate",
            bc::Navigate::new(bc::NavigateParameters::new(
                s("ctx-1"),
                s("https://example.com/"),
                Some(bc::ReadinessState::Interactive),
            )),
        ),
        cmd(
            "browsingContext.print",
            bc::Print::new(bc::PrintParameters::new(
                s("ctx-1"),
                Some(true),
                Some(bc::PrintMarginParameters::new(
                    Some(1.0),
                    Some(1.5),
                    Some(2.0),
                    Some(0.5),
                )),
                Some(bc::PrintParametersOrientation::Landscape),
                Some(bc::PrintPageParameters::new(Some(29.7), Some(21.0))),
                Some(vec![
                    bc::JsUintOrText::JsUint(1),
                    bc::JsUintOrText::Text(s("3-5")),
                ]),
                Some(1.5),
                Some(false),
            )),
        ),
        cmd(
            "browsingContext.reload",
            bc::Reload::new(bc::ReloadParameters::new(
                s("ctx-1"),
                Some(true),
                Some(bc::ReadinessState::Complete),
            )),
        ),
        cmd(
            "browsingContext.setBypassCSP",
            bc::SetBypassCSP::new(bc::SetBypassCSPParameters::new(Some(true), ctxs(), None)),
        ),
        cmd(
            "browsingContext.setBypassCSP (reset)",
            bc::SetBypassCSP::new(bc::SetBypassCSPParameters::new(None, None, ucs())),
        ),
        cmd(
            "browsingContext.setViewport",
            bc::SetViewport::new(bc::SetViewportParameters::new(
                Some(s("ctx-1")),
                Some(bc::Viewport::new(1280, 720)),
                Some(2.0),
                None,
            )),
        ),
        cmd(
            "browsingContext.setViewport (user contexts)",
            bc::SetViewport::new(bc::SetViewportParameters::new(
                None,
                Some(bc::Viewport::new(1280, 720)),
                None,
                ucs(),
            )),
        ),
        cmd(
            "browsingContext.startScreencast",
            bc::StartScreencast::new(bc::StartScreencastParameters::new(
                s("ctx-1"),
                Some(s("/tmp/videos")),
                Some(s("video/webm")),
                Some(bc::MediaTrackConstraints::new(
                    Some(1280),
                    Some(720),
                    Some(30),
                )),
                Some(true),
            )),
        ),
        cmd(
            "browsingContext.stopScreencast",
            bc::StopScreencast::new(bc::StopScreencastParameters::new(s("screencast-1"))),
        ),
        cmd(
            "browsingContext.traverseHistory",
            bc::TraverseHistory::new(bc::TraverseHistoryParameters::new(s("ctx-1"), -1)),
        ),
        // emulation
        cmd(
            "emulation.setForcedColorsModeThemeOverride",
            em::SetForcedColorsModeThemeOverride::new(
                em::SetForcedColorsModeThemeOverrideParameters::new(
                    Some(em::ForcedColorsModeTheme::Dark),
                    ctxs(),
                    None,
                ),
            ),
        ),
        cmd(
            "emulation.setGeolocationOverride (coordinates)",
            em::SetGeolocationOverride::new(
                em::SetGeolocationOverrideParameters::with_coordinates(
                    Some(em::GeolocationCoordinates {
                        latitude: -33.86,
                        longitude: 151.21,
                        accuracy: 10.0,
                        altitude: Some(58.0),
                        altitude_accuracy: Some(2.0),
                        heading: Some(90.0),
                        speed: Some(1.5),
                    }),
                    ctxs(),
                    None,
                ),
            ),
        ),
        cmd(
            "emulation.setGeolocationOverride (reset)",
            em::SetGeolocationOverride::new(
                em::SetGeolocationOverrideParameters::with_coordinates(None, None, ucs()),
            ),
        ),
        cmd(
            "emulation.setGeolocationOverride (error)",
            em::SetGeolocationOverride::new(em::SetGeolocationOverrideParameters::with_error(
                em::GeolocationPositionError::position_unavailable(),
                ctxs(),
                None,
            )),
        ),
        cmd(
            "emulation.setLocaleOverride",
            em::SetLocaleOverride::new(em::SetLocaleOverrideParameters {
                locale: Some(s("de-DE")),
                contexts: ctxs(),
                user_contexts: None,
            }),
        ),
        cmd(
            "emulation.setMediaFeaturesOverride",
            em::SetMediaFeaturesOverride::new(em::SetMediaFeaturesOverrideParameters::new(
                Some(all_media_features()),
                ctxs(),
                None,
            )),
        ),
        cmd(
            "emulation.setNetworkConditions",
            em::SetNetworkConditions::new(em::SetNetworkConditionsParameters::new(
                Some(em::NetworkConditions::Offline),
                None,
                ucs(),
            )),
        ),
        cmd(
            "emulation.setScreenOrientationOverride",
            em::SetScreenOrientationOverride::new(em::SetScreenOrientationOverrideParameters {
                screen_orientation: Some(em::ScreenOrientation {
                    natural: em::ScreenOrientationNatural::Landscape,
                    orientation_type: em::ScreenOrientationType::PortraitSecondary,
                }),
                contexts: ctxs(),
                user_contexts: None,
            }),
        ),
        cmd(
            "emulation.setScreenSettingsOverride",
            em::SetScreenSettingsOverride::new(em::SetScreenSettingsOverrideParameters::new(
                Some(em::ScreenArea::new(1920, 1080)),
                ctxs(),
                None,
            )),
        ),
        cmd(
            "emulation.setScriptingEnabled",
            em::SetScriptingEnabled::new(em::SetScriptingEnabledParameters::new(
                Some(em::ScriptingEnabled::Disabled),
                ctxs(),
                None,
            )),
        ),
        cmd(
            "emulation.setScrollbarTypeOverride",
            em::SetScrollbarTypeOverride::new(em::SetScrollbarTypeOverrideParameters::new(
                Some(em::ScrollbarType::Overlay),
                ctxs(),
                None,
            )),
        ),
        cmd(
            "emulation.setTextLayoutModeOverride",
            em::SetTextLayoutModeOverride::new(em::SetTextLayoutModeOverrideParameters::new(
                Some(em::TextLayoutMode::Mobile),
                ctxs(),
                None,
            )),
        ),
        cmd(
            "emulation.setTimezoneOverride",
            em::SetTimezoneOverride::new(em::SetTimezoneOverrideParameters {
                timezone: Some(s("Europe/Berlin")),
                contexts: None,
                user_contexts: ucs(),
            }),
        ),
        cmd(
            "emulation.setTouchOverride",
            em::SetTouchOverride::new(em::SetTouchOverrideParameters::new(Some(5), ctxs(), None)),
        ),
        cmd(
            "emulation.setUserAgentOverride",
            em::SetUserAgentOverride::new(em::SetUserAgentOverrideParameters::new(
                Some(s("TestAgent/1.0")),
                ctxs(),
                None,
            )),
        ),
        cmd(
            "emulation.setViewportMetaOverride",
            em::SetViewportMetaOverride::new(em::SetViewportMetaOverrideParameters::new(
                Some(em::ViewportMeta::Enabled),
                ctxs(),
                None,
            )),
        ),
        // input
        cmd(
            "input.performActions",
            inp::PerformActions::new(inp::PerformActionsParameters::new(
                s("ctx-1"),
                vec![
                    inp::SourceActions::NoneSourceActions(inp::NoneSourceActions::new(
                        s("none-1"),
                        vec![inp::PauseAction::new(Some(10))],
                    )),
                    inp::SourceActions::KeySourceActions(inp::KeySourceActions::new(
                        s("key-1"),
                        vec![
                            inp::KeySourceAction::PauseAction(inp::PauseAction::new(None)),
                            inp::KeySourceAction::KeyDownAction(inp::KeyDownAction::new(s("a"))),
                            inp::KeySourceAction::KeyUpAction(inp::KeyUpAction::new(s("a"))),
                        ],
                    )),
                    inp::SourceActions::PointerSourceActions(inp::PointerSourceActions::new(
                        s("pointer-1"),
                        Some(inp::PointerParameters::new(Some(inp::PointerType::Pen))),
                        vec![
                            inp::PointerSourceAction::PauseAction(inp::PauseAction::new(Some(5))),
                            inp::PointerSourceAction::PointerMoveAction(
                                inp::PointerMoveAction::new(
                                    1.5,
                                    2.5,
                                    Some(100),
                                    Some(inp::Origin::ElementOrigin(inp::ElementOrigin::new(
                                        shared_ref("node-1"),
                                    ))),
                                    pointer_props(),
                                ),
                            ),
                            inp::PointerSourceAction::PointerMoveAction(
                                inp::PointerMoveAction::new(
                                    0.0,
                                    0.0,
                                    None,
                                    Some(inp::Origin::Pointer(s("pointer"))),
                                    inp::PointerCommonProperties::new(
                                        None, None, None, None, None, None, None,
                                    ),
                                ),
                            ),
                            inp::PointerSourceAction::PointerDownAction(
                                inp::PointerDownAction::new(0, pointer_props()),
                            ),
                            inp::PointerSourceAction::PointerUpAction(inp::PointerUpAction::new(0)),
                        ],
                    )),
                    inp::SourceActions::WheelSourceActions(inp::WheelSourceActions::new(
                        s("wheel-1"),
                        vec![
                            inp::WheelSourceAction::PauseAction(inp::PauseAction::new(None)),
                            inp::WheelSourceAction::WheelScrollAction(inp::WheelScrollAction::new(
                                10,
                                20,
                                -5,
                                15,
                                Some(50),
                                Some(inp::Origin::Viewport(s("viewport"))),
                            )),
                        ],
                    )),
                ],
            )),
        ),
        cmd(
            "input.performActions (pointer source without parameters)",
            inp::PerformActions::new(inp::PerformActionsParameters::new(
                s("ctx-1"),
                vec![inp::SourceActions::PointerSourceActions(
                    inp::PointerSourceActions::new(
                        s("pointer-1"),
                        None,
                        vec![inp::PointerSourceAction::PointerUpAction(
                            inp::PointerUpAction::new(0),
                        )],
                    ),
                )],
            )),
        ),
        cmd(
            "input.releaseActions",
            inp::ReleaseActions::new(inp::ReleaseActionsParameters::new(s("ctx-1"))),
        ),
        cmd(
            "input.setFiles",
            inp::SetFiles::new(inp::SetFilesParameters::new(
                s("ctx-1"),
                shared_ref("node-1"),
                vec![s("/tmp/a.txt"), s("/tmp/b.txt")],
            )),
        ),
        // network
        cmd(
            "network.addDataCollector",
            net::AddDataCollector::new(net::AddDataCollectorParameters {
                data_types: vec![s("request"), s("response")],
                max_encoded_data_size: 1024,
                collector_type: Some(s("blob")),
                contexts: ctxs(),
                user_contexts: None,
            }),
        ),
        cmd(
            "network.addIntercept",
            net::AddIntercept::new(net::AddInterceptParameters::new(
                vec![
                    net::InterceptPhase::BeforeRequestSent,
                    net::InterceptPhase::ResponseStarted,
                    net::InterceptPhase::AuthRequired,
                ],
                ctxs(),
                Some(vec![
                    net::UrlPattern::UrlPatternPattern(net::UrlPatternPattern::new(
                        Some(s("https")),
                        Some(s("example.com")),
                        Some(s("443")),
                        Some(s("/api")),
                        Some(s("q=1")),
                    )),
                    net::UrlPattern::UrlPatternString(net::UrlPatternString::new(s(
                        "https://example.com/*",
                    ))),
                ]),
            )),
        ),
        cmd(
            "network.continueRequest",
            net::ContinueRequest::new(net::ContinueRequestParameters::new(
                s("request-1"),
                Some(string_bytes("body")),
                Some(vec![net::CookieHeader::new(s("a"), base64_bytes("Yg=="))]),
                Some(vec![header("X-Test")]),
                Some(s("POST")),
                Some(s("https://example.com/other")),
            )),
        ),
        cmd(
            "network.continueResponse",
            net::ContinueResponse::new(net::ContinueResponseParameters::new(
                s("request-1"),
                Some(vec![set_cookie_header()]),
                Some(net::AuthCredentials::new(s("user"), s("pass"))),
                Some(vec![header("X-Test")]),
                Some(s("OK")),
                Some(200),
            )),
        ),
        cmd(
            "network.continueWithAuth (credentials)",
            net::ContinueWithAuth::new(net::ContinueWithAuthParameters::new(
                s("request-1"),
                Some(net::ContinueWithAuthOption::Credentials(
                    net::ContinueWithAuthCredentials::new(
                        s("provideCredentials"),
                        net::AuthCredentials::new(s("user"), s("pass")),
                    ),
                )),
            )),
        ),
        cmd(
            "network.continueWithAuth (default)",
            net::ContinueWithAuth::new(net::ContinueWithAuthParameters::new(
                s("request-1"),
                Some(net::ContinueWithAuthOption::NoCredentials(
                    net::ContinueWithAuthNoCredentials::new(net::NoCredentialsAction::Default),
                )),
            )),
        ),
        cmd(
            "network.continueWithAuth (cancel)",
            net::ContinueWithAuth::new(net::ContinueWithAuthParameters::new(
                s("request-1"),
                Some(net::ContinueWithAuthOption::NoCredentials(
                    net::ContinueWithAuthNoCredentials::new(net::NoCredentialsAction::Cancel),
                )),
            )),
        ),
        cmd(
            "network.disownData",
            net::DisownData::new(net::DisownDataParameters {
                data_type: s("response"),
                collector: s("collector-1"),
                request: s("request-1"),
            }),
        ),
        cmd(
            "network.failRequest",
            net::FailRequest::new(net::FailRequestParameters::new(s("request-1"))),
        ),
        cmd(
            "network.getData",
            net::GetData::new(net::GetDataParameters {
                data_type: s("request"),
                collector: Some(s("collector-1")),
                disown: Some(true),
                request: s("request-1"),
            }),
        ),
        cmd(
            "network.provideResponse",
            net::ProvideResponse::new(net::ProvideResponseParameters::new(
                s("request-1"),
                Some(base64_bytes("aGk=")),
                Some(vec![set_cookie_header()]),
                Some(vec![header("Content-Type")]),
                Some(s("Not Found")),
                Some(404),
            )),
        ),
        cmd(
            "network.removeDataCollector",
            net::RemoveDataCollector::new(net::RemoveDataCollectorParameters {
                collector: s("collector-1"),
            }),
        ),
        cmd(
            "network.removeIntercept",
            net::RemoveIntercept::new(net::RemoveInterceptParameters::new(s("intercept-1"))),
        ),
        cmd(
            "network.setCacheBehavior",
            net::SetCacheBehavior::new(net::SetCacheBehaviorParameters::new(
                net::CacheBehavior::Bypass,
                ctxs(),
            )),
        ),
        cmd(
            "network.setCacheBehavior (default)",
            net::SetCacheBehavior::new(net::SetCacheBehaviorParameters::new(
                net::CacheBehavior::Default,
                None,
            )),
        ),
        cmd(
            "network.setExtraHeaders",
            net::SetExtraHeaders::new(net::SetExtraHeadersParameters {
                headers: vec![header("X-Extra")],
                contexts: ctxs(),
                user_contexts: None,
            }),
        ),
        // script
        cmd(
            "script.addPreloadScript",
            sc::AddPreloadScript::new(sc::AddPreloadScriptParameters::new(
                s("() => {}"),
                Some(vec![channel_value()]),
                ctxs(),
                None,
                Some(s("sandbox-1")),
            )),
        ),
        cmd(
            "script.callFunction (every LocalValue kind)",
            sc::CallFunction::new(sc::CallFunctionParameters::new(
                s("(...args) => args.length"),
                true,
                sc::Target::RealmTarget(sc::RealmTarget::new(s("realm-1"))),
                Some(local_values()),
                Some(sc::ResultOwnership::Root),
                Some(serialization_options()),
                Some(string_local("this")),
                Some(true),
            )),
        ),
        cmd(
            "script.disown",
            sc::Disown::new(sc::DisownParameters::new(
                vec![s("handle-1")],
                sc::Target::ContextTarget(sc::ContextTarget::new(s("ctx-1"), Some(s("sandbox-1")))),
            )),
        ),
        cmd(
            "script.evaluate",
            sc::Evaluate::new(sc::EvaluateParameters::new(
                s("1 + 1"),
                sc::Target::ContextTarget(sc::ContextTarget::new(s("ctx-1"), None)),
                false,
                Some(sc::ResultOwnership::None),
                Some(serialization_options()),
                Some(false),
            )),
        ),
        cmd(
            "script.getRealms",
            sc::GetRealms::new(sc::GetRealmsParameters::new(
                Some(s("ctx-1")),
                Some(sc::RealmType::DedicatedWorker),
            )),
        ),
        cmd(
            "script.removePreloadScript",
            sc::RemovePreloadScript::new(sc::RemovePreloadScriptParameters::new(s("script-1"))),
        ),
        // session
        cmd("session.end", ses::End::new(EmptyParams::new())),
        cmd(
            "session.new",
            ses::New::new(ses::NewParameters::new(ses::CapabilitiesRequest {
                always_match: Some(capability_request("chrome")),
                first_match: Some(vec![capability_request("firefox")]),
            })),
        ),
        cmd("session.status", ses::Status::new(EmptyParams::new())),
        cmd(
            "session.subscribe",
            ses::Subscribe::new(ses::SubscribeParameters::new(
                vec![s("browsingContext.load"), s("network")],
                ctxs(),
                None,
            )),
        ),
        cmd(
            "session.subscribe (user contexts)",
            ses::Subscribe::new(ses::SubscribeParameters::new(
                vec![s("log.entryAdded")],
                None,
                ucs(),
            )),
        ),
        cmd(
            "session.unsubscribe (by id)",
            ses::Unsubscribe::new(ses::UnsubscribeParameters::UnsubscribeByIDRequest(
                ses::UnsubscribeByIDRequest::new(vec![s("subscription-1")]),
            )),
        ),
        cmd(
            "session.unsubscribe (by attributes)",
            ses::Unsubscribe::new(ses::UnsubscribeParameters::UnsubscribeByAttributesRequest(
                ses::UnsubscribeByAttributesRequest::new(vec![s("browsingContext.load")]),
            )),
        ),
        // storage
        cmd(
            "storage.deleteCookies",
            st::DeleteCookies::new(st::DeleteCookiesParameters::new(
                Some(st::CookieFilter::new(
                    Some(s("session")),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    Extensible::new(),
                )),
                Some(st::PartitionDescriptor::BrowsingContextPartitionDescriptor(
                    st::BrowsingContextPartitionDescriptor::new(s("ctx-1")),
                )),
            )),
        ),
        cmd(
            "storage.getCookies",
            st::GetCookies::new(st::GetCookiesParameters::new(
                Some(st::CookieFilter::new(
                    Some(s("session")),
                    Some(string_bytes("abc")),
                    Some(s("example.com")),
                    Some(s("/")),
                    Some(10),
                    Some(true),
                    Some(false),
                    Some(net::SameSite::Strict),
                    Some(1_800_000_000),
                    Extensible::new(),
                )),
                Some(st::PartitionDescriptor::StorageKeyPartitionDescriptor(
                    st::StorageKeyPartitionDescriptor::new(
                        Some(s("user-ctx-1")),
                        Some(s("https://example.com")),
                    ),
                )),
            )),
        ),
        cmd(
            "storage.setCookie",
            st::SetCookie::new(st::SetCookieParameters::new(
                st::PartialCookie::new(
                    s("session"),
                    string_bytes("abc"),
                    s("example.com"),
                    Some(s("/")),
                    Some(true),
                    Some(true),
                    Some(net::SameSite::None),
                    Some(1_800_000_000),
                    Extensible::new(),
                ),
                None,
            )),
        ),
        // webExtension
        cmd(
            "webExtension.install (path)",
            wx::Install::new(wx::InstallParameters::new(
                wx::ExtensionData::ExtensionPath(wx::ExtensionPath::new(s("/tmp/extension"))),
            )),
        ),
        cmd(
            "webExtension.install (archive)",
            wx::Install::new(wx::InstallParameters::new(
                wx::ExtensionData::ExtensionArchivePath(wx::ExtensionArchivePath::new(s(
                    "/tmp/extension.zip",
                ))),
            )),
        ),
        cmd(
            "webExtension.install (base64)",
            wx::Install::new(wx::InstallParameters::new(
                wx::ExtensionData::ExtensionBase64Encoded(wx::ExtensionBase64Encoded::new(s(
                    "UEsDBA==",
                ))),
            )),
        ),
        cmd(
            "webExtension.uninstall",
            wx::Uninstall::new(wx::UninstallParameters::new(s("extension-1"))),
        ),
    ]
}

#[test]
fn commands_conform_to_remote_cddl() {
    let spec = spec();
    let cases = command_cases();
    let mut failures = Vec::new();
    for case in &cases {
        let method = case.json["method"].as_str().unwrap_or("<no method>");
        let root = format!("{{ id: js-uint, {}, Extensible }}", rule_for_method(method));
        if let Err(e) = validate(&spec.remote, spec.remote_src, &root, &case.json) {
            failures.push(format!(
                "  {}\n    error: {}\n    sent:  {}",
                case.label,
                short(&e),
                case.json
            ));
        }
    }
    report("commands", failures, cases.len());
}

#[test]
fn every_spec_command_is_covered() {
    let spec = spec();
    let covered: Vec<String> = command_cases()
        .iter()
        .filter_map(|c| c.json["method"].as_str().map(str::to_string))
        .collect();
    let missing: Vec<String> = spec_methods(spec.remote_src)
        .into_iter()
        .filter(|m| !covered.contains(m))
        .collect();
    assert!(
        missing.is_empty(),
        "spec commands without a conformance case: {missing:?}"
    );
}

// ---------------------------------------------------------------------------
// Command results and events (local end input)
// ---------------------------------------------------------------------------

struct Fixture {
    /// Spec rule the JSON must satisfy, e.g. `browsingContext.NavigateResult`.
    rule: String,
    label: String,
    json: Value,
    round_trip: RoundTrip,
}

fn result<T: DeserializeOwned + Serialize>(rule: &str, json: Value) -> Fixture {
    Fixture {
        rule: s(rule),
        label: s(rule),
        json,
        round_trip: round_trip::<T>,
    }
}

fn result_labeled<T: DeserializeOwned + Serialize>(
    rule: &str,
    label: &str,
    json: Value,
) -> Fixture {
    Fixture {
        rule: s(rule),
        label: format!("{rule} ({label})"),
        json,
        round_trip: round_trip::<T>,
    }
}

/// An event; `json` is `{ "method": ..., "params": ... }`.
fn event<T: DeserializeOwned + Serialize>(label: &str, json: Value) -> Fixture {
    let method = json["method"].as_str().expect("event method");
    Fixture {
        rule: rule_for_method(method),
        label: s(label),
        json,
        round_trip: round_trip::<T>,
    }
}

fn client_window_info() -> Value {
    json!({
        "active": true,
        "clientWindow": "window-1",
        "height": 600,
        "state": "normal",
        "width": 800,
        "x": -10,
        "y": 20
    })
}

fn context_info() -> Value {
    json!({
        "children": [{
            "children": null,
            "clientWindow": "window-1",
            "context": "ctx-2",
            "originalOpener": null,
            "url": "about:blank",
            "userContext": "default",
            "parent": "ctx-1"
        }],
        "clientWindow": "window-1",
        "context": "ctx-1",
        "originalOpener": "ctx-0",
        "url": "https://example.com/",
        "userContext": "default",
        "parent": null
    })
}

fn node() -> Value {
    json!({
        "type": "node",
        "sharedId": "node-1",
        "handle": "handle-1",
        "internalId": "internal-1",
        "value": {
            "nodeType": 1,
            "childNodeCount": 1,
            "attributes": { "id": "main" },
            "children": [{ "type": "node", "sharedId": "node-2" }],
            "localName": "div",
            "mode": "open",
            "namespaceURI": "http://www.w3.org/1999/xhtml",
            "nodeValue": "",
            "shadowRoot": null
        }
    })
}

fn stack_trace() -> Value {
    json!({
        "callFrames": [{
            "columnNumber": 4,
            "functionName": "f",
            "lineNumber": 10,
            "url": "https://example.com/app.js"
        }]
    })
}

/// One `script.RemoteValue` of every kind.
fn remote_values() -> Vec<(&'static str, Value)> {
    vec![
        ("undefined", json!({ "type": "undefined" })),
        ("null", json!({ "type": "null" })),
        ("string", json!({ "type": "string", "value": "hi" })),
        ("number", json!({ "type": "number", "value": 1.5 })),
        ("number -0", json!({ "type": "number", "value": "-0" })),
        ("number NaN", json!({ "type": "number", "value": "NaN" })),
        ("boolean", json!({ "type": "boolean", "value": false })),
        (
            "bigint",
            json!({ "type": "bigint", "value": "9007199254740993" }),
        ),
        (
            "symbol",
            json!({ "type": "symbol", "handle": "h", "internalId": "i" }),
        ),
        (
            "array",
            json!({ "type": "array", "handle": "h", "internalId": "i",
                    "value": [{ "type": "string", "value": "a" }] }),
        ),
        (
            "object",
            json!({ "type": "object", "handle": "h", "internalId": "i",
                    "value": [["key", { "type": "number", "value": 1 }],
                              [{ "type": "string", "value": "k" }, { "type": "null" }]] }),
        ),
        ("function", json!({ "type": "function", "handle": "h" })),
        (
            "regexp",
            json!({ "type": "regexp", "value": { "pattern": "a+", "flags": "g" }, "handle": "h" }),
        ),
        (
            "date",
            json!({ "type": "date", "value": "2026-11-27T00:00:00.000Z", "internalId": "i" }),
        ),
        (
            "map",
            json!({ "type": "map", "value": [["k", { "type": "boolean", "value": true }]] }),
        ),
        (
            "set",
            json!({ "type": "set", "value": [{ "type": "string", "value": "a" }] }),
        ),
        ("weakmap", json!({ "type": "weakmap", "handle": "h" })),
        ("weakset", json!({ "type": "weakset", "handle": "h" })),
        ("generator", json!({ "type": "generator", "handle": "h" })),
        ("error", json!({ "type": "error", "handle": "h" })),
        ("proxy", json!({ "type": "proxy", "handle": "h" })),
        ("promise", json!({ "type": "promise", "handle": "h" })),
        ("typedarray", json!({ "type": "typedarray", "handle": "h" })),
        (
            "arraybuffer",
            json!({ "type": "arraybuffer", "handle": "h" }),
        ),
        ("nodelist", json!({ "type": "nodelist", "value": [node()] })),
        (
            "htmlcollection",
            json!({ "type": "htmlcollection", "value": [node()] }),
        ),
        ("node", node()),
        (
            "window",
            json!({ "type": "window", "value": { "context": "ctx-1" }, "handle": "h" }),
        ),
    ]
}

fn request_data() -> Value {
    json!({
        "request": "request-1",
        "url": "https://example.com/",
        "method": "GET",
        "headers": [
            { "name": "Accept", "value": { "type": "string", "value": "*/*" } },
            { "name": "X-Bin", "value": { "type": "base64", "value": "AA==" } }
        ],
        "cookies": [cookie()],
        "headersSize": 120,
        "bodySize": null,
        "destination": "document",
        "initiatorType": null,
        "timings": {
            "timeOrigin": 1.0, "requestTime": 2.0, "redirectStart": 0.0, "redirectEnd": 0.0,
            "fetchStart": 3.0, "dnsStart": 4.0, "dnsEnd": 5.0, "connectStart": 6.0,
            "connectEnd": 7.0, "tlsStart": 6.5, "requestStart": 8.0, "responseStart": 9.0,
            "responseEnd": 10.0
        }
    })
}

fn cookie() -> Value {
    json!({
        "name": "session",
        "value": { "type": "string", "value": "abc" },
        "domain": "example.com",
        "path": "/",
        "size": 10,
        "httpOnly": true,
        "secure": true,
        "sameSite": "lax",
        "expiry": 1800000000
    })
}

fn response_data() -> Value {
    json!({
        "url": "https://example.com/",
        "protocol": "h2",
        "status": 401,
        "statusText": "Unauthorized",
        "fromCache": false,
        "headers": [{ "name": "Content-Type", "value": { "type": "string", "value": "text/html" } }],
        "mimeType": "text/html",
        "bytesReceived": 512,
        "headersSize": 100,
        "bodySize": 412,
        "content": { "size": 412 },
        "authChallenges": [{ "scheme": "Basic", "realm": "test" }]
    })
}

/// `network.BaseParameters` merged with event-specific fields.
fn network_params(extra: Value) -> Value {
    let mut params = json!({
        "context": "ctx-1",
        "isBlocked": true,
        "navigation": "nav-1",
        "redirectCount": 0,
        "request": request_data(),
        "timestamp": 1700000000000u64,
        "userContext": "default",
        "intercepts": ["intercept-1"]
    });
    params
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    params
}

fn navigation_info() -> Value {
    json!({
        "context": "ctx-1",
        "navigation": "nav-1",
        "timestamp": 1700000000000u64,
        "url": "https://example.com/",
        "userContext": "default"
    })
}

fn source() -> Value {
    json!({ "realm": "realm-1", "context": "ctx-1", "userContext": "default" })
}

fn result_fixtures() -> Vec<Fixture> {
    let empty = |rule: &str| result::<EmptyResult>(rule, json!({}));
    let mut fixtures = vec![
        // browser
        empty("browser.CloseResult"),
        result::<b::CreateUserContextResult>(
            "browser.CreateUserContextResult",
            json!({ "userContext": "user-ctx-1" }),
        ),
        result::<b::GetClientWindowsResult>(
            "browser.GetClientWindowsResult",
            json!({ "clientWindows": [client_window_info()] }),
        ),
        result::<b::GetUserContextsResult>(
            "browser.GetUserContextsResult",
            json!({ "userContexts": [{ "userContext": "default" }] }),
        ),
        empty("browser.RemoveUserContextResult"),
        result::<b::ClientWindowInfo>("browser.SetClientWindowStateResult", client_window_info()),
        empty("browser.SetDownloadBehaviorResult"),
        // browsingContext
        empty("browsingContext.ActivateResult"),
        result::<bc::CaptureScreenshotResult>(
            "browsingContext.CaptureScreenshotResult",
            json!({ "data": "iVBORw0KGgo=" }),
        ),
        empty("browsingContext.CloseResult"),
        result::<bc::CreateResult>(
            "browsingContext.CreateResult",
            json!({ "context": "ctx-1", "userContext": "user-ctx-1" }),
        ),
        result::<bc::GetTreeResult>(
            "browsingContext.GetTreeResult",
            json!({ "contexts": [context_info()] }),
        ),
        empty("browsingContext.HandleUserPromptResult"),
        result::<bc::LocateNodesResult>(
            "browsingContext.LocateNodesResult",
            json!({ "nodes": [node()] }),
        ),
        result::<bc::NavigateResult>(
            "browsingContext.NavigateResult",
            json!({ "navigation": "nav-1", "url": "https://example.com/" }),
        ),
        result::<bc::PrintResult>("browsingContext.PrintResult", json!({ "data": "JVBERi0=" })),
        result::<bc::NavigateResult>(
            "browsingContext.ReloadResult",
            json!({ "navigation": null, "url": "https://example.com/" }),
        ),
        empty("browsingContext.SetBypassCSPResult"),
        empty("browsingContext.SetViewportResult"),
        result::<bc::StartScreencastResult>(
            "browsingContext.StartScreencastResult",
            json!({ "screencast": "screencast-1", "path": "/tmp/videos/1.webm" }),
        ),
        result::<bc::StopScreencastResult>(
            "browsingContext.StopScreencastResult",
            json!({ "path": "/tmp/videos/1.webm", "error": "disk full" }),
        ),
        result::<bc::TraverseHistoryResult>("browsingContext.TraverseHistoryResult", json!({})),
        // input
        empty("input.PerformActionsResult"),
        empty("input.ReleaseActionsResult"),
        empty("input.SetFilesResult"),
        // network
        result::<net::AddDataCollectorResult>(
            "network.AddDataCollectorResult",
            json!({ "collector": "collector-1" }),
        ),
        result::<net::AddInterceptResult>(
            "network.AddInterceptResult",
            json!({ "intercept": "intercept-1" }),
        ),
        empty("network.ContinueRequestResult"),
        empty("network.ContinueResponseResult"),
        empty("network.ContinueWithAuthResult"),
        empty("network.DisownDataResult"),
        empty("network.FailRequestResult"),
        result_labeled::<net::GetDataResult>(
            "network.GetDataResult",
            "base64",
            json!({ "bytes": { "type": "base64", "value": "aGk=" } }),
        ),
        result_labeled::<net::GetDataResult>(
            "network.GetDataResult",
            "string",
            json!({ "bytes": { "type": "string", "value": "hi" } }),
        ),
        empty("network.ProvideResponseResult"),
        empty("network.RemoveDataCollectorResult"),
        empty("network.RemoveInterceptResult"),
        empty("network.SetCacheBehaviorResult"),
        empty("network.SetExtraHeadersResult"),
        // script
        result::<sc::AddPreloadScriptResult>(
            "script.AddPreloadScriptResult",
            json!({ "script": "script-1" }),
        ),
        empty("script.DisownResult"),
        result::<sc::GetRealmsResult>(
            "script.GetRealmsResult",
            json!({ "realms": [
                { "realm": "r1", "origin": "https://example.com", "type": "window",
                  "context": "ctx-1", "userContext": "default", "sandbox": "sb" },
                { "realm": "r2", "origin": "null", "type": "dedicated-worker", "owners": ["r1"] },
                { "realm": "r3", "origin": "null", "type": "shared-worker" },
                { "realm": "r4", "origin": "null", "type": "service-worker" },
                { "realm": "r5", "origin": "null", "type": "worker" },
                { "realm": "r6", "origin": "null", "type": "paint-worklet" },
                { "realm": "r7", "origin": "null", "type": "audio-worklet" },
                { "realm": "r8", "origin": "null", "type": "worklet" }
            ] }),
        ),
        empty("script.RemovePreloadScriptResult"),
        result_labeled::<sc::EvaluationResult>(
            "script.EvaluationResult",
            "exception",
            json!({
                "type": "exception",
                "exceptionDetails": {
                    "columnNumber": 1,
                    "exception": { "type": "error", "handle": "h" },
                    "lineNumber": 2,
                    "stackTrace": stack_trace(),
                    "text": "ReferenceError: x is not defined"
                },
                "realm": "realm-1"
            }),
        ),
        result::<sc::EvaluateResult>(
            "script.EvaluateResult",
            json!({ "type": "success", "result": { "type": "number", "value": 2 }, "realm": "realm-1" }),
        ),
        result::<sc::CallFunctionResult>(
            "script.CallFunctionResult",
            json!({ "type": "success", "result": { "type": "undefined" }, "realm": "realm-1" }),
        ),
        // session
        empty("session.EndResult"),
        result::<ses::NewResult>(
            "session.NewResult",
            json!({
                "sessionId": "session-1",
                "capabilities": {
                    "acceptInsecureCerts": false,
                    "browserName": "chrome",
                    "browserVersion": "158.0.8127.88",
                    "platformName": "linux",
                    "setWindowRect": true,
                    "userAgent": "Mozilla/5.0",
                    "proxy": { "proxyType": "direct" },
                    "unhandledPromptBehavior": { "default": "dismiss" },
                    "webSocketUrl": "ws://localhost:4444/session/session-1",
                    "goog:chromeOptions": { "debuggerAddress": "localhost:1234" }
                }
            }),
        ),
        result::<ses::StatusResult>(
            "session.StatusResult",
            json!({ "ready": true, "message": "ready" }),
        ),
        result::<ses::SubscribeResult>(
            "session.SubscribeResult",
            json!({ "subscription": "subscription-1" }),
        ),
        empty("session.UnsubscribeResult"),
        // storage
        result::<st::DeleteCookiesResult>(
            "storage.DeleteCookiesResult",
            json!({ "partitionKey": { "userContext": "default", "sourceOrigin": "https://example.com" } }),
        ),
        result::<st::GetCookiesResult>(
            "storage.GetCookiesResult",
            json!({ "cookies": [cookie()], "partitionKey": { "userContext": "default" } }),
        ),
        result::<st::SetCookieResult>("storage.SetCookieResult", json!({ "partitionKey": {} })),
        // webExtension
        result::<wx::InstallResult>(
            "webExtension.InstallResult",
            json!({ "extension": "extension-1" }),
        ),
        empty("webExtension.UninstallResult"),
    ];
    // Every emulation command returns EmptyResult.
    for method in spec_methods(spec().remote_src) {
        if method.starts_with("emulation.") {
            fixtures.push(empty(&format!("{}Result", rule_for_method(&method))));
        }
    }
    // script.EvaluationResult with every kind of RemoteValue.
    for (kind, value) in remote_values() {
        fixtures.push(result_labeled::<sc::EvaluationResult>(
            "script.EvaluationResult",
            kind,
            json!({ "type": "success", "result": value, "realm": "realm-1" }),
        ));
    }
    fixtures
}

fn event_fixtures() -> Vec<Fixture> {
    vec![
        event::<bc::ContextCreated>(
            "browsingContext.contextCreated",
            json!({ "method": "browsingContext.contextCreated",
                    "params": { "hasPlannedNavigation": true,
                                "children": null, "clientWindow": "window-1", "context": "ctx-1",
                                "originalOpener": null, "url": "about:blank",
                                "userContext": "default", "parent": "ctx-0" } }),
        ),
        event::<bc::ContextDestroyed>(
            "browsingContext.contextDestroyed",
            json!({ "method": "browsingContext.contextDestroyed", "params": context_info() }),
        ),
        event::<bc::DomContentLoaded>(
            "browsingContext.domContentLoaded",
            json!({ "method": "browsingContext.domContentLoaded", "params": navigation_info() }),
        ),
        event::<bc::DownloadEnd>(
            "browsingContext.downloadEnd (complete)",
            json!({ "method": "browsingContext.downloadEnd",
                    "params": { "status": "complete", "download": "download-1",
                                "filepath": "/tmp/downloads/file.zip",
                                "context": "ctx-1", "navigation": "nav-1", "timestamp": 1,
                                "url": "https://example.com/file.zip", "userContext": "default" } }),
        ),
        event::<bc::DownloadEnd>(
            "browsingContext.downloadEnd (canceled)",
            json!({ "method": "browsingContext.downloadEnd",
                    "params": { "status": "canceled", "download": "download-1",
                                "context": "ctx-1", "navigation": null, "timestamp": 1,
                                "url": "https://example.com/file.zip" } }),
        ),
        event::<bc::DownloadWillBegin>(
            "browsingContext.downloadWillBegin",
            json!({ "method": "browsingContext.downloadWillBegin",
                    "params": { "download": "download-1", "suggestedFilename": "file.zip",
                                "context": "ctx-1", "navigation": "nav-1", "timestamp": 1,
                                "url": "https://example.com/file.zip", "userContext": "default" } }),
        ),
        event::<bc::FragmentNavigated>(
            "browsingContext.fragmentNavigated",
            json!({ "method": "browsingContext.fragmentNavigated", "params": navigation_info() }),
        ),
        event::<bc::HistoryUpdated>(
            "browsingContext.historyUpdated",
            json!({ "method": "browsingContext.historyUpdated",
                    "params": { "context": "ctx-1", "timestamp": 1,
                                "url": "https://example.com/#a", "userContext": "default" } }),
        ),
        event::<bc::Load>(
            "browsingContext.load",
            json!({ "method": "browsingContext.load", "params": navigation_info() }),
        ),
        event::<bc::NavigationAborted>(
            "browsingContext.navigationAborted",
            json!({ "method": "browsingContext.navigationAborted", "params": navigation_info() }),
        ),
        event::<bc::NavigationCommitted>(
            "browsingContext.navigationCommitted",
            json!({ "method": "browsingContext.navigationCommitted", "params": navigation_info() }),
        ),
        event::<bc::NavigationFailed>(
            "browsingContext.navigationFailed",
            json!({ "method": "browsingContext.navigationFailed", "params": navigation_info() }),
        ),
        event::<bc::NavigationStarted>(
            "browsingContext.navigationStarted",
            json!({ "method": "browsingContext.navigationStarted", "params": navigation_info() }),
        ),
        event::<bc::UserPromptClosed>(
            "browsingContext.userPromptClosed",
            json!({ "method": "browsingContext.userPromptClosed",
                    "params": { "context": "ctx-1", "accepted": true, "type": "prompt",
                                "userContext": "default", "userText": "answer" } }),
        ),
        event::<bc::UserPromptOpened>(
            "browsingContext.userPromptOpened",
            json!({ "method": "browsingContext.userPromptOpened",
                    "params": { "context": "ctx-1", "handler": "ignore", "message": "Leave?",
                                "type": "beforeunload", "userContext": "default",
                                "defaultValue": "" } }),
        ),
        event::<inp::FileDialogOpened>(
            "input.fileDialogOpened",
            json!({ "method": "input.fileDialogOpened",
                    "params": { "context": "ctx-1", "userContext": "default",
                                "element": { "sharedId": "node-1" }, "multiple": true } }),
        ),
        event::<lg::EntryAdded>(
            "log.entryAdded (console)",
            json!({ "method": "log.entryAdded",
                    "params": { "type": "console", "method": "log",
                                "args": [{ "type": "string", "value": "hi" }],
                                "level": "info", "source": source(), "text": "hi",
                                "timestamp": 1, "stackTrace": stack_trace() } }),
        ),
        event::<lg::EntryAdded>(
            "log.entryAdded (javascript)",
            json!({ "method": "log.entryAdded",
                    "params": { "type": "javascript", "level": "error", "source": source(),
                                "text": "Uncaught Error", "timestamp": 1 } }),
        ),
        event::<lg::EntryAdded>(
            "log.entryAdded (generic)",
            json!({ "method": "log.entryAdded",
                    "params": { "type": "other", "level": "warn",
                                "source": { "realm": "realm-1" }, "text": null,
                                "timestamp": 1 } }),
        ),
        event::<net::AuthRequired>(
            "network.authRequired",
            json!({ "method": "network.authRequired",
                    "params": network_params(json!({ "response": response_data() })) }),
        ),
        event::<net::BeforeRequestSent>(
            "network.beforeRequestSent",
            json!({ "method": "network.beforeRequestSent",
                    "params": network_params(json!({ "initiator": {
                        "columnNumber": 1, "lineNumber": 2, "request": "request-0",
                        "stackTrace": stack_trace(), "type": "script" } })) }),
        ),
        event::<net::FetchError>(
            "network.fetchError",
            json!({ "method": "network.fetchError",
                    "params": network_params(json!({ "errorText": "net::ERR_FAILED" })) }),
        ),
        event::<net::ResponseCompleted>(
            "network.responseCompleted",
            json!({ "method": "network.responseCompleted",
                    "params": network_params(json!({ "response": response_data() })) }),
        ),
        event::<net::ResponseStarted>(
            "network.responseStarted",
            json!({ "method": "network.responseStarted",
                    "params": network_params(json!({ "response": response_data() })) }),
        ),
        event::<sc::Message>(
            "script.message",
            json!({ "method": "script.message",
                    "params": { "channel": "channel-1",
                                "data": { "type": "string", "value": "hi" },
                                "source": source() } }),
        ),
        event::<sc::RealmCreated>(
            "script.realmCreated",
            json!({ "method": "script.realmCreated",
                    "params": { "realm": "realm-1", "origin": "https://example.com",
                                "type": "window", "context": "ctx-1" } }),
        ),
        event::<sc::RealmDestroyed>(
            "script.realmDestroyed",
            json!({ "method": "script.realmDestroyed", "params": { "realm": "realm-1" } }),
        ),
    ]
}

fn check_fixtures(fixtures: &[Fixture], is_event: bool) -> Vec<String> {
    let spec = spec();
    let mut failures = Vec::new();
    for fixture in fixtures {
        // Events are checked inside the `Event` envelope a browser sends.
        let (root, wire) = if is_event {
            let mut wire = fixture.json.clone();
            wire.as_object_mut()
                .unwrap()
                .insert(s("type"), json!("event"));
            (
                format!("{{ type: \"event\", {}, Extensible }}", fixture.rule),
                wire,
            )
        } else {
            (fixture.rule.clone(), fixture.json.clone())
        };
        if let Err(e) = validate(&spec.local, spec.local_src, &root, &wire) {
            failures.push(format!(
                "  {}\n    fixture is not spec-valid (fix the test): {}",
                fixture.label,
                short(&e)
            ));
            continue;
        }
        match (fixture.round_trip)(&fixture.json) {
            Err(e) => failures.push(format!(
                "  {}\n    {e}\n    received: {}",
                fixture.label, fixture.json
            )),
            Ok(back) => {
                let mut lost = Vec::new();
                lossy_fields(&fixture.json, &back, "", &mut lost);
                if !lost.is_empty() {
                    failures.push(format!(
                        "  {}\n    round trip lost data: {}\n    received:     {}\n    round-tripped: {}",
                        fixture.label,
                        lost.join(", "),
                        fixture.json,
                        back
                    ));
                }
            }
        }
    }
    failures
}

#[test]
fn results_conform_to_local_cddl() {
    let fixtures = result_fixtures();
    report("results", check_fixtures(&fixtures, false), fixtures.len());
}

#[test]
fn events_conform_to_local_cddl() {
    let fixtures = event_fixtures();
    report("events", check_fixtures(&fixtures, true), fixtures.len());
}

#[test]
fn every_spec_result_is_covered() {
    let covered: Vec<String> = result_fixtures().into_iter().map(|f| f.rule).collect();
    let missing: Vec<String> = spec_result_rules(spec().local_src)
        .into_iter()
        .filter(|r| !covered.contains(r))
        .collect();
    assert!(
        missing.is_empty(),
        "spec results without a conformance fixture: {missing:?}"
    );
}

#[test]
fn every_spec_event_is_covered() {
    let covered: Vec<String> = event_fixtures()
        .iter()
        .filter_map(|f| f.json["method"].as_str().map(str::to_string))
        .collect();
    let missing: Vec<String> = spec_methods(spec().local_src)
        .into_iter()
        .filter(|m| !covered.contains(m))
        .collect();
    assert!(
        missing.is_empty(),
        "spec events without a conformance fixture: {missing:?}"
    );
}

/// Error codes the spec's prose defines and uses but its CDDL `ErrorCode` omits.
const PROSE_ONLY_ERROR_CODES: &[&str] = &["no such client window"];

#[test]
fn prose_only_error_codes_parse() {
    let cddl_codes = spec_error_codes(spec().local_src);
    for code in PROSE_ONLY_ERROR_CODES {
        assert!(
            !cddl_codes.iter().any(|c| c == code),
            "`{code}` is now in the CDDL ErrorCode list; remove it from PROSE_ONLY_ERROR_CODES"
        );
        assert!(
            !matches!(
                serde_json::from_value::<ErrorCode>(json!(code)),
                Ok(ErrorCode::Other(_)) | Err(_)
            ),
            "`{code}` is not a known ErrorCode variant"
        );
    }
}

#[test]
fn every_spec_error_code_parses() {
    let spec = spec();
    let codes = spec_error_codes(spec.local_src);
    assert!(codes.len() > 20, "failed to read ErrorCode from local.cddl");
    let mut failures = Vec::new();
    for code in &codes {
        let response = json!({ "type": "error", "id": 1, "error": code, "message": "m",
                               "stacktrace": "at f()" });
        if let Err(e) = validate(&spec.local, spec.local_src, "ErrorResponse", &response) {
            failures.push(format!(
                "  {code}: fixture is not spec-valid: {}",
                short(&e)
            ));
            continue;
        }
        match round_trip::<ErrorResponse>(&response) {
            Err(e) => failures.push(format!("  {code}: {e}")),
            Ok(back) => {
                let mut lost = Vec::new();
                lossy_fields(&response, &back, "", &mut lost);
                if !lost.is_empty() {
                    failures.push(format!("  {code}: round trip lost {}", lost.join(", ")));
                }
            }
        }
        match serde_json::from_value::<ErrorCode>(json!(code)) {
            Ok(ErrorCode::Other(_)) | Err(_) => {
                failures.push(format!("  {code}: not a known ErrorCode variant"))
            }
            Ok(_) => {}
        }
    }
    report("error codes", failures, codes.len());
}

// ---------------------------------------------------------------------------
// The validator itself must not be vacuous.
// ---------------------------------------------------------------------------

#[test]
fn validator_rejects_invalid_messages() {
    let spec = spec();
    let navigate = "{ id: js-uint, browsingContext.Navigate, Extensible }";
    let bad_commands = [
        (
            "unknown key",
            json!({ "id": 1, "method": "browsingContext.navigate",
                                "params": { "context": "c", "url": "u", "bogus": 1 } }),
        ),
        (
            "snake_case key",
            json!({ "id": 1, "method": "browsingContext.navigate",
                                   "params": { "context": "c", "url": "u", "read_state": "none" } }),
        ),
        (
            "wrong type",
            json!({ "id": 1, "method": "browsingContext.navigate",
                               "params": { "context": 1, "url": "u" } }),
        ),
        (
            "missing field",
            json!({ "id": 1, "method": "browsingContext.navigate",
                                  "params": { "context": "c" } }),
        ),
        (
            "bad enum",
            json!({ "id": 1, "method": "browsingContext.navigate",
                             "params": { "context": "c", "url": "u", "wait": "done" } }),
        ),
        (
            "wrong method",
            json!({ "id": 1, "method": "browsingContext.reload",
                                 "params": { "context": "c", "url": "u" } }),
        ),
    ];
    for (what, json) in bad_commands {
        assert!(
            validate(&spec.remote, spec.remote_src, navigate, &json).is_err(),
            "validator accepted a command with a {what}: {json}"
        );
    }
    assert!(
        validate(
            &spec.local,
            spec.local_src,
            "browsingContext.NavigateResult",
            &json!({ "url": "u" })
        )
        .is_err(),
        "validator accepted a result with a missing field"
    );
    assert!(
        validate(&spec.local, spec.local_src, "nope.Missing", &json!({})).is_err(),
        "an undefined rule must be an error"
    );
}
