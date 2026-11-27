//! Validation against the spec's CDDL (`cddl/*.cddl`), shared by the conformance
//! suite and the live tests' traffic checks.

use std::sync::OnceLock;

use cddl::ast::CDDL;
use cddl::validator::{Validator, json::JSONValidator};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

// ---------------------------------------------------------------------------
// CDDL loading and validation
// ---------------------------------------------------------------------------

/// Rewrites applied to the spec CDDL before validation. They work around bugs
/// in the `cddl` crate (0.10.7) and only loosen the schema. Each `from` string
/// must still occur in the spec, so a stale patch is noticed.
pub const VALIDATOR_WORKAROUNDS: &[(&str, &str, &str)] = &[
    (
        "js-int = -9007199254740991..9007199254740991",
        "js-int = int",
        "the validator rejects every value against a range with a negative lower bound",
    ),
    (
        "(js-uint .ge 1)",
        "(uint .ge 1)",
        "the validator can't apply `.ge` to a type that is itself a range",
    ),
    (
        "network.CollectorType .default \"blob\"",
        "network.CollectorType",
        "the validator rejects every value when `.default` follows a literal-valued rule",
    ),
    (
        "  bytes: network.BytesValue,",
        "  \"bytes\": network.BytesValue,",
        "the validator reads a bare `bytes` key as the CDDL `bytes` type",
    ),
    (
        "  text: text / null,",
        "  \"text\": text / null,",
        "inside an included group (log.BaseLogEntry), the validator reads a bare `text` key as the CDDL `text` type",
    ),
];

pub struct Spec {
    pub remote_src: &'static str,
    pub local_src: &'static str,
    pub remote: CDDL<'static>,
    pub local: CDDL<'static>,
}

pub fn spec() -> &'static Spec {
    static SPEC: OnceLock<Spec> = OnceLock::new();
    SPEC.get_or_init(|| {
        let read = |name: &str| {
            let path = format!("{}/cddl/{name}", env!("CARGO_MANIFEST_DIR"));
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path}: {e}"))
        };
        let (mut remote, mut local) = (read("remote.cddl"), read("local.cddl"));
        // JSON has one number type, so `1` is a valid `float`; the validator only
        // accepts a JSON integer for `int`. Accept integers wherever the spec says
        // `float` (this only loosens the schema).
        remote = float_as_number(&remote);
        local = float_as_number(&local);
        for (from, to, why) in VALIDATOR_WORKAROUNDS {
            assert!(
                remote.contains(from) || local.contains(from),
                "validator workaround no longer applies ({why}); remove it: {from}"
            );
            remote = remote.replace(from, to);
            local = local.replace(from, to);
        }
        let remote_src: &'static str = Box::leak(remote.into_boxed_str());
        let local_src: &'static str = Box::leak(local.into_boxed_str());
        Spec {
            remote_src,
            local_src,
            remote: cddl::cddl_from_str(remote_src, true).expect("parse remote.cddl"),
            local: cddl::cddl_from_str(local_src, true).expect("parse local.cddl"),
        }
    })
}

/// Validate `json` against the CDDL type expression `root`, which is spliced in
/// as the document's first (root) rule so the spec is parsed only once.
pub fn validate(
    doc: &CDDL<'static>,
    doc_src: &str,
    root: &str,
    json: &Value,
) -> Result<(), String> {
    // A typo in a rule name must fail loudly instead of validating against nothing.
    for name in rule_refs(root) {
        let definition = format!("{name} =");
        if !doc_src
            .lines()
            .any(|l| l.trim_start().starts_with(&definition))
        {
            return Err(format!("rule `{name}` is not defined in the spec"));
        }
    }
    let src: &'static str = Box::leak(format!("root = {root}").into_boxed_str());
    let root_rule =
        cddl::cddl_from_str(src, true).map_err(|e| format!("bad root `{root}`: {e}"))?;
    let mut doc = doc.clone();
    doc.rules.insert(0, root_rule.rules[0].clone());
    let mut validator = JSONValidator::new(&doc, json.clone(), None);
    validator.validate().map_err(|e| e.to_string())
}

/// Dotted rule names (e.g. `browsingContext.Navigate`) referenced by a root expression.
pub fn rule_refs(root: &str) -> Vec<&str> {
    root.split(|c: char| !(c.is_alphanumeric() || c == '.' || c == '-'))
        .filter(|t| t.contains('.') && t.chars().next().is_some_and(char::is_alphabetic))
        .collect()
}

/// `browsingContext.navigate` -> `browsingContext.Navigate`
pub fn rule_for_method(method: &str) -> String {
    let (module, name) = method.split_once('.').expect("method has a module");
    let mut chars = name.chars();
    let first = chars.next().expect("non-empty name").to_ascii_uppercase();
    format!("{module}.{first}{}", chars.as_str())
}

/// Quoted `method: "x.y"` values in a CDDL file.
pub fn spec_methods(src: &str) -> Vec<String> {
    let mut out: Vec<String> = src
        .split("method: \"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_string)
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Names of rules like `browsingContext.NavigateResult = ...`.
pub fn spec_result_rules(src: &str) -> Vec<String> {
    let mut out: Vec<String> = src
        .lines()
        .filter_map(|l| l.split_once(" = ").map(|(name, _)| name.trim()))
        .filter(|name| name.contains('.') && name.ends_with("Result"))
        .map(str::to_string)
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The quoted alternatives of `ErrorCode = "a" / "b" / ...`.
pub fn spec_error_codes(src: &str) -> Vec<String> {
    let start = src.find("ErrorCode = ").expect("ErrorCode rule");
    let body = &src[start..];
    let end = body.find("\n\n").unwrap_or(body.len());
    body[..end]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// Fields in `original` that are missing or changed in `round_tripped`.
/// A field whose value is `null` may be dropped, since an absent optional
/// field and an explicit `null` mean the same thing to this crate's types.
pub fn lossy_fields(original: &Value, round_tripped: &Value, path: &str, out: &mut Vec<String>) {
    match (original, round_tripped) {
        (Value::Object(a), Value::Object(b)) => {
            for (key, value) in a {
                let child = format!("{path}/{key}");
                match b.get(key) {
                    None if value.is_null() => {}
                    None => out.push(format!("{child}: dropped")),
                    Some(other) => lossy_fields(value, other, &child, out),
                }
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                lossy_fields(x, y, &format!("{path}/{i}"), out);
            }
        }
        (Value::Number(a), Value::Number(b)) if a.as_f64() == b.as_f64() => {}
        (a, b) if a == b => {}
        (a, b) => out.push(format!("{path}: {a} became {b}")),
    }
}

pub fn short(e: &str) -> String {
    let lines: Vec<&str> = e.lines().filter(|l| !l.trim().is_empty()).take(4).collect();
    lines.join("\n      ")
}

pub fn report(kind: &str, failures: Vec<String>, total: usize) {
    if !failures.is_empty() {
        panic!(
            "{} of {total} {kind} failed:\n\n{}\n",
            failures.len(),
            failures.join("\n\n")
        );
    }
    println!("{total} {kind} conform to the spec");
}

pub type RoundTrip = fn(&Value) -> Result<Value, String>;

/// Deserialize into `T` and serialize back.
pub fn round_trip<T: DeserializeOwned + Serialize>(json: &Value) -> Result<Value, String> {
    let parsed: T =
        serde_json::from_value(json.clone()).map_err(|e| format!("deserialize: {e}"))?;
    serde_json::to_value(&parsed).map_err(|e| format!("serialize: {e}"))
}

/// The crate type a command's result is parsed into (as declared by
/// `define_command!`), as a round-trip function.
pub fn result_round_trip(method: &str) -> Option<RoundTrip> {
    use webdriverbidi::model::browser as b;
    use webdriverbidi::model::browsing_context as bc;
    use webdriverbidi::model::network as net;
    use webdriverbidi::model::result::EmptyResult;
    use webdriverbidi::model::script as sc;
    use webdriverbidi::model::session as ses;
    use webdriverbidi::model::storage as st;
    use webdriverbidi::model::web_extension as wx;
    Some(match method {
        "browser.createUserContext" => round_trip::<b::CreateUserContextResult>,
        "browser.getClientWindows" => round_trip::<b::GetClientWindowsResult>,
        "browser.getUserContexts" => round_trip::<b::GetUserContextsResult>,
        "browser.setClientWindowState" => round_trip::<b::ClientWindowInfo>,
        "browsingContext.captureScreenshot" => round_trip::<bc::CaptureScreenshotResult>,
        "browsingContext.create" => round_trip::<bc::CreateResult>,
        "browsingContext.getTree" => round_trip::<bc::GetTreeResult>,
        "browsingContext.locateNodes" => round_trip::<bc::LocateNodesResult>,
        "browsingContext.navigate" | "browsingContext.reload" => round_trip::<bc::NavigateResult>,
        "browsingContext.print" => round_trip::<bc::PrintResult>,
        "browsingContext.startScreencast" => round_trip::<bc::StartScreencastResult>,
        "browsingContext.stopScreencast" => round_trip::<bc::StopScreencastResult>,
        "browsingContext.traverseHistory" => round_trip::<bc::TraverseHistoryResult>,
        "network.addDataCollector" => round_trip::<net::AddDataCollectorResult>,
        "network.addIntercept" => round_trip::<net::AddInterceptResult>,
        "network.getData" => round_trip::<net::GetDataResult>,
        "script.addPreloadScript" => round_trip::<sc::AddPreloadScriptResult>,
        "script.callFunction" | "script.evaluate" => round_trip::<sc::EvaluationResult>,
        "script.getRealms" => round_trip::<sc::GetRealmsResult>,
        "session.new" => round_trip::<ses::NewResult>,
        "session.status" => round_trip::<ses::StatusResult>,
        "session.subscribe" => round_trip::<ses::SubscribeResult>,
        "storage.deleteCookies" => round_trip::<st::DeleteCookiesResult>,
        "storage.getCookies" => round_trip::<st::GetCookiesResult>,
        "storage.setCookie" => round_trip::<st::SetCookieResult>,
        "webExtension.install" => round_trip::<wx::InstallResult>,
        m if m.contains('.') => round_trip::<EmptyResult>,
        _ => return None,
    })
}

/// The crate type an event (`{ "method", "params" }`) is parsed into.
pub fn event_round_trip(method: &str) -> Option<RoundTrip> {
    use webdriverbidi::model::browsing_context as bc;
    use webdriverbidi::model::input as inp;
    use webdriverbidi::model::log as lg;
    use webdriverbidi::model::network as net;
    use webdriverbidi::model::script as sc;
    Some(match method {
        "browsingContext.contextCreated" => round_trip::<bc::ContextCreated>,
        "browsingContext.contextDestroyed" => round_trip::<bc::ContextDestroyed>,
        "browsingContext.domContentLoaded" => round_trip::<bc::DomContentLoaded>,
        "browsingContext.downloadEnd" => round_trip::<bc::DownloadEnd>,
        "browsingContext.downloadWillBegin" => round_trip::<bc::DownloadWillBegin>,
        "browsingContext.fragmentNavigated" => round_trip::<bc::FragmentNavigated>,
        "browsingContext.historyUpdated" => round_trip::<bc::HistoryUpdated>,
        "browsingContext.load" => round_trip::<bc::Load>,
        "browsingContext.navigationAborted" => round_trip::<bc::NavigationAborted>,
        "browsingContext.navigationCommitted" => round_trip::<bc::NavigationCommitted>,
        "browsingContext.navigationFailed" => round_trip::<bc::NavigationFailed>,
        "browsingContext.navigationStarted" => round_trip::<bc::NavigationStarted>,
        "browsingContext.userPromptClosed" => round_trip::<bc::UserPromptClosed>,
        "browsingContext.userPromptOpened" => round_trip::<bc::UserPromptOpened>,
        "input.fileDialogOpened" => round_trip::<inp::FileDialogOpened>,
        "log.entryAdded" => round_trip::<lg::EntryAdded>,
        "network.authRequired" => round_trip::<net::AuthRequired>,
        "network.beforeRequestSent" => round_trip::<net::BeforeRequestSent>,
        "network.fetchError" => round_trip::<net::FetchError>,
        "network.responseCompleted" => round_trip::<net::ResponseCompleted>,
        "network.responseStarted" => round_trip::<net::ResponseStarted>,
        "script.message" => round_trip::<sc::Message>,
        "script.realmCreated" => round_trip::<sc::RealmCreated>,
        "script.realmDestroyed" => round_trip::<sc::RealmDestroyed>,
        _ => return None,
    })
}

/// Problems found in one session's traffic.
#[derive(Default)]
pub struct TrafficReport {
    /// Bugs in this crate: a command that doesn't match the spec, or a received
    /// message that fails to parse or loses data in the crate's types.
    pub crate_bugs: Vec<String>,
    /// Received messages that don't match the spec: the browser deviates from it.
    pub spec_deviations: Vec<String>,
    pub messages: usize,
}

/// Check every message of a session: commands against `remote.cddl`; responses,
/// errors and events against `local.cddl` and through the crate's types.
pub fn check_traffic(
    traffic: &[(webdriverbidi::session::MessageDirection, Value)],
) -> TrafficReport {
    use std::collections::HashMap;
    use webdriverbidi::model::error::ErrorResponse;
    use webdriverbidi::session::MessageDirection;

    let spec = spec();
    let mut report = TrafficReport {
        messages: traffic.len(),
        ..Default::default()
    };
    let mut sent_methods: HashMap<u64, String> = HashMap::new();
    for (direction, message) in traffic {
        let compact = || {
            let text = message.to_string();
            if text.len() > 600 {
                format!("{}…", &text[..600])
            } else {
                text
            }
        };
        let method = message["method"].as_str().unwrap_or_default().to_string();
        if *direction == MessageDirection::Sent {
            if let Some(id) = message["id"].as_u64() {
                sent_methods.insert(id, method.clone());
            }
            let root = format!(
                "{{ id: js-uint, {}, Extensible }}",
                rule_for_method(&method)
            );
            if let Err(e) = validate(&spec.remote, spec.remote_src, &root, message) {
                report.crate_bugs.push(format!(
                    "sent {method} doesn't match the spec: {}\n      {}",
                    short(&e),
                    compact()
                ));
            }
            continue;
        }
        // What the message is, the rule it must satisfy, the JSON to validate and
        // the JSON to round-trip through the crate's type.
        let (what, root, payload, parse): (String, String, Value, Option<RoundTrip>) =
            match message["type"].as_str() {
                Some("success") => {
                    let id = message["id"].as_u64().unwrap_or_default();
                    let Some(command) = sent_methods.get(&id) else {
                        report
                            .crate_bugs
                            .push(format!("response to unknown id {id}: {}", compact()));
                        continue;
                    };
                    (
                        format!("result of {command}"),
                        format!("{}Result", rule_for_method(command)),
                        message["result"].clone(),
                        result_round_trip(command),
                    )
                }
                Some("error") => (
                    "error response".into(),
                    "ErrorResponse".into(),
                    message.clone(),
                    Some(round_trip::<ErrorResponse> as RoundTrip),
                ),
                Some("event") => {
                    let mut event = message.clone();
                    event.as_object_mut().map(|o| o.remove("type"));
                    (
                        format!("event {method}"),
                        format!(
                            "{{ type: \"event\", {}, Extensible }}",
                            rule_for_method(&method)
                        ),
                        event,
                        event_round_trip(&method),
                    )
                }
                _ => {
                    report
                        .spec_deviations
                        .push(format!("unrecognized message: {}", compact()));
                    continue;
                }
            };
        let to_validate = if message["type"] == "event" {
            message
        } else {
            &payload
        };
        let conforms = match validate(&spec.local, spec.local_src, &root, to_validate) {
            Ok(()) => true,
            Err(e) => {
                report.spec_deviations.push(format!(
                    "{what} doesn't match the spec: {}\n      {}",
                    short(&e),
                    compact()
                ));
                false
            }
        };
        let Some(parse) = parse else {
            report
                .crate_bugs
                .push(format!("{what}: no crate type to parse it into"));
            continue;
        };
        match parse(&payload) {
            // Failing to parse breaks users whether or not the browser conforms.
            Err(e) => report
                .crate_bugs
                .push(format!("{what} fails to parse: {e}\n      {}", compact())),
            Ok(back) => {
                let mut lost = Vec::new();
                lossy_fields(&payload, &back, "", &mut lost);
                if !lost.is_empty() {
                    let line = format!(
                        "{what} loses data in the crate's types: {}\n      {}",
                        lost.join(", "),
                        compact()
                    );
                    // Fields outside the spec are the browser's, not the crate's, problem.
                    if conforms {
                        report.crate_bugs.push(line)
                    } else {
                        report.spec_deviations.push(line)
                    }
                }
            }
        }
    }
    report
}

/// Replace the CDDL type `float` (as a whole word) with `number` (`int / float`).
pub fn float_as_number(cddl: &str) -> String {
    let mut out = String::with_capacity(cddl.len());
    let mut rest = cddl;
    while let Some(i) = rest.find("float") {
        let before = rest[..i].chars().next_back();
        let after = rest[i + 5..].chars().next();
        let is_word = |c: Option<char>| {
            c.is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        };
        out.push_str(&rest[..i]);
        out.push_str(if is_word(before) || is_word(after) {
            "float"
        } else {
            "number"
        });
        rest = &rest[i + 5..];
    }
    out.push_str(rest);
    out
}
