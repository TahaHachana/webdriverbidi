use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::browser::UserContext;
use crate::model::browsing_context::BrowsingContext;
use crate::model::common::{Extensible, JsUint};
use crate::model::result::EmptyResult;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScriptCommand {
    AddPreloadScript(AddPreloadScript),
    CallFunction(CallFunction),
    Disown(Disown),
    Evaluate(Evaluate),
    GetRealms(GetRealms),
    RemovePreloadScript(RemovePreloadScript),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ScriptResult {
    AddPreloadScriptResult(AddPreloadScriptResult),
    EvaluateResult(EvaluateResult),
    GetRealmsResult(GetRealmsResult),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ScriptEvent {
    Message(Message),
    RealmCreated(RealmCreated),
    RealmDestroyed(RealmDestroyed),
}

pub type Channel = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: ChannelProperties,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelProperties {
    pub channel: Channel,
    #[serde(
        rename = "serializationOptions",
        skip_serializing_if = "Option::is_none"
    )]
    pub serialization_options: Option<SerializationOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ownership: Option<ResultOwnership>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum EvaluateResult {
    EvaluateResultSuccess(EvaluateResultSuccess),
    EvaluateResultException(EvaluateResultException),
    EmptyResult(EmptyResult),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EvaluateResultSuccess {
    #[serde(rename = "type")]
    pub result_type: String,
    pub result: RemoteValue,
    pub realm: Realm,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EvaluateResultException {
    #[serde(rename = "type")]
    pub result_type: String,
    #[serde(rename = "exceptionDetails")]
    pub exception_details: ExceptionDetails,
    pub realm: Realm,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExceptionDetails {
    #[serde(rename = "columnNumber")]
    pub column_number: JsUint,
    pub exception: RemoteValue,
    #[serde(rename = "lineNumber")]
    pub line_number: JsUint,
    #[serde(rename = "stackTrace")]
    pub stack_trace: StackTrace,
    pub text: String,
}

pub type Handle = String;

pub type InternalId = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum LocalValue {
    RemoteReference(RemoteReference),
    PrimitiveProtocolValue(PrimitiveProtocolValue),
    ChannelValue(ChannelValue),
    ArrayLocalValue(ArrayLocalValue),
    DateLocalValue(DateLocalValue),
    MapLocalValue(MapLocalValue),
    ObjectLocalValue(ObjectLocalValue),
    RegExpLocalValue(RegExpLocalValue),
    SetLocalValue(SetLocalValue),
}

pub type ListLocalValue = Vec<LocalValue>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ArrayLocalValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: ListLocalValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DateLocalValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MappingLocalValue(pub Vec<(LocalValueOrText, LocalValue)>);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LocalValueOrText {
    LocalValue(LocalValue),
    Text(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MapLocalValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: MappingLocalValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectLocalValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: MappingLocalValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegExpValue {
    pub pattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegExpLocalValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: RegExpValue,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetLocalValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: ListLocalValue,
}

pub type PreloadScript = String;

pub type Realm = String;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum PrimitiveProtocolValue {
    StringValue(StringValue),
    NumberValue(NumberValue),
    BooleanValue(BooleanValue),
    BigIntValue(BigIntValue),
    NullValue(NullValue),
    UndefinedValue(UndefinedValue),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UndefinedValue {
    #[serde(rename = "type")]
    pub value_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NullValue {
    #[serde(rename = "type")]
    pub value_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StringValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

impl StringValue {
    pub fn new(value: String) -> Self {
        Self {
            value_type: "string".to_string(),
            value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SpecialNumber {
    NaN,
    #[serde(rename = "-0")]
    NegativeZero,
    Infinity,
    #[serde(rename = "-Infinity")]
    NegativeInfinity,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NumberValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: NumberOrSpecialNumber,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum NumberOrSpecialNumber {
    Number(f64),
    SpecialNumber(SpecialNumber),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BooleanValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BigIntValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RealmInfo {
    WindowRealmInfo(WindowRealmInfo),
    DedicatedWorkerRealmInfo(DedicatedWorkerRealmInfo),
    SharedWorkerRealmInfo(SharedWorkerRealmInfo),
    ServiceWorkerRealmInfo(ServiceWorkerRealmInfo),
    WorkerRealmInfo(WorkerRealmInfo),
    PaintWorkletRealmInfo(PaintWorkletRealmInfo),
    AudioWorkletRealmInfo(AudioWorkletRealmInfo),
    WorkletRealmInfo(WorkletRealmInfo),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BaseRealmInfo {
    pub realm: Realm,
    pub origin: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowRealmInfo {
    #[serde(flatten)]
    pub base: BaseRealmInfo,
    #[serde(rename = "type")]
    pub realm_type: String,
    pub context: BrowsingContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DedicatedWorkerRealmInfo {
    #[serde(flatten)]
    pub base: BaseRealmInfo,
    #[serde(rename = "type")]
    pub realm_type: String,
    pub owners: Vec<Realm>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SharedWorkerRealmInfo {
    #[serde(flatten)]
    pub base: BaseRealmInfo,
    #[serde(rename = "type")]
    pub realm_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServiceWorkerRealmInfo {
    #[serde(flatten)]
    pub base: BaseRealmInfo,
    #[serde(rename = "type")]
    pub realm_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkerRealmInfo {
    #[serde(flatten)]
    pub base: BaseRealmInfo,
    #[serde(rename = "type")]
    pub realm_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PaintWorkletRealmInfo {
    #[serde(flatten)]
    pub base: BaseRealmInfo,
    #[serde(rename = "type")]
    pub realm_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AudioWorkletRealmInfo {
    #[serde(flatten)]
    pub base: BaseRealmInfo,
    #[serde(rename = "type")]
    pub realm_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkletRealmInfo {
    #[serde(flatten)]
    pub base: BaseRealmInfo,
    #[serde(rename = "type")]
    pub realm_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RealmType {
    #[serde(rename = "window")]
    Window,
    #[serde(rename = "dedicated-worker")]
    DedicatedWorker,
    #[serde(rename = "shared-worker")]
    SharedWorker,
    #[serde(rename = "service-worker")]
    ServiceWorker,
    #[serde(rename = "worker")]
    Worker,
    #[serde(rename = "paint-worklet")]
    PaintWorklet,
    #[serde(rename = "audio-worklet")]
    AudioWorklet,
    #[serde(rename = "worklet")]
    Worklet,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RemoteReference {
    SharedReference(SharedReference),
    RemoteObjectReference(RemoteObjectReference),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SharedReference {
    #[serde(rename = "sharedId")]
    pub shared_id: SharedId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoteObjectReference {
    pub handle: Handle,
    #[serde(rename = "sharedId", skip_serializing_if = "Option::is_none")]
    pub shared_id: Option<SharedId>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RemoteValue {
    PrimitiveProtocolValue(PrimitiveProtocolValue),
    SymbolRemoteValue(SymbolRemoteValue),
    ArrayRemoteValue(ArrayRemoteValue),
    ObjectRemoteValue(ObjectRemoteValue),
    FunctionRemoteValue(FunctionRemoteValue),
    RegExpRemoteValue(RegExpRemoteValue),
    DateRemoteValue(DateRemoteValue),
    MapRemoteValue(MapRemoteValue),
    SetRemoteValue(SetRemoteValue),
    WeakMapRemoteValue(WeakMapRemoteValue),
    WeakSetRemoteValue(WeakSetRemoteValue),
    GeneratorRemoteValue(GeneratorRemoteValue),
    ErrorRemoteValue(ErrorRemoteValue),
    ProxyRemoteValue(ProxyRemoteValue),
    PromiseRemoteValue(PromiseRemoteValue),
    TypedArrayRemoteValue(TypedArrayRemoteValue),
    ArrayBufferRemoteValue(ArrayBufferRemoteValue),
    NodeListRemoteValue(NodeListRemoteValue),
    HTMLCollectionRemoteValue(HTMLCollectionRemoteValue),
    NodeRemoteValue(NodeRemoteValue),
    WindowProxyRemoteValue(WindowProxyRemoteValue),
}

pub type ListRemoteValue = Vec<RemoteValue>;

pub type MappingRemoteValue = Vec<(RemoteValueOrText, RemoteValue)>;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RemoteValueOrText {
    RemoteValue(RemoteValue),
    Text(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SymbolRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArrayRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<ListRemoteValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<MappingRemoteValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FunctionRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegExpRemoteValue {
    #[serde(flatten)]
    pub local_value: RegExpLocalValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DateRemoteValue {
    #[serde(flatten)]
    pub local_value: DateLocalValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MapRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<MappingRemoteValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<ListRemoteValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeakMapRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeakSetRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GeneratorRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxyRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromiseRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TypedArrayRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArrayBufferRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeListRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<ListRemoteValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HTMLCollectionRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<ListRemoteValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(rename = "sharedId", skip_serializing_if = "Option::is_none")]
    pub shared_id: Option<SharedId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Box<NodeProperties>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeProperties {
    #[serde(rename = "nodeType")]
    pub node_type: JsUint,
    #[serde(rename = "childNodeCount")]
    pub child_node_count: JsUint,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<NodeRemoteValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "localName")]
    pub local_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "mode")]
    pub mode: Option<NodePropertiesMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "namespaceURI")]
    pub namespace_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "nodeValue")]
    pub node_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "shadowRoot")]
    pub shadow_root: Option<NodeRemoteValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // Match JSON values ("open" / "closed")
pub enum NodePropertiesMode {
    Open,
    Closed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowProxyRemoteValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: WindowProxyProperties,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<Handle>,
    #[serde(rename = "internalId", skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<InternalId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowProxyProperties {
    pub context: BrowsingContext,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ResultOwnership {
    Root,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializationOptions {
    #[serde(rename = "maxDomDepth", skip_serializing_if = "Option::is_none")]
    pub max_dom_depth: Option<JsUint>,
    #[serde(rename = "maxObjectDepth", skip_serializing_if = "Option::is_none")]
    pub max_object_depth: Option<JsUint>,
    #[serde(rename = "includeShadowTree", skip_serializing_if = "Option::is_none")]
    pub include_shadow_tree: Option<IncludeShadowTree>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum IncludeShadowTree {
    None,
    Open,
    All,
}

pub type SharedId = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct StackFrame {
    #[serde(rename = "columnNumber")]
    pub column_number: JsUint,
    #[serde(rename = "functionName")]
    pub function_name: String,
    #[serde(rename = "lineNumber")]
    pub line_number: JsUint,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StackTrace {
    #[serde(rename = "callFrames")]
    pub call_frames: Vec<StackFrame>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    pub realm: Realm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<BrowsingContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RealmTarget {
    pub realm: Realm,
}

impl RealmTarget {
    pub fn new(realm: Realm) -> Self {
        Self { realm }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextTarget {
    pub context: BrowsingContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<String>,
}

impl ContextTarget {
    pub fn new(context: BrowsingContext, sandbox: Option<String>) -> Self {
        Self { context, sandbox }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Target {
    ContextTarget(ContextTarget),
    RealmTarget(RealmTarget),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPreloadScript {
    pub method: String,
    pub params: AddPreloadScriptParameters,
}

impl AddPreloadScript {
    pub fn new(params: AddPreloadScriptParameters) -> Self {
        Self {
            method: "script.addPreloadScript".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPreloadScriptParameters {
    #[serde(rename = "functionDeclaration")]
    pub function_declaration: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<ChannelValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "userContexts")]
    pub user_contexts: Option<Vec<UserContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox: Option<String>,
}

impl AddPreloadScriptParameters {
    pub fn new(
        function_declaration: String,
        arguments: Option<Vec<ChannelValue>>,
        contexts: Option<Vec<BrowsingContext>>,
        user_contexts: Option<Vec<UserContext>>,
        sandbox: Option<String>,
    ) -> Self {
        Self {
            function_declaration,
            arguments,
            contexts,
            user_contexts,
            sandbox,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddPreloadScriptResult {
    pub script: PreloadScript,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disown {
    pub method: String,
    pub params: DisownParameters,
}

impl Disown {
    pub fn new(params: DisownParameters) -> Self {
        Self {
            method: "script.disown".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisownParameters {
    pub handles: Vec<Handle>,
    pub target: Target,
}

impl DisownParameters {
    pub fn new(handles: Vec<Handle>, target: Target) -> Self {
        Self { handles, target }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallFunction {
    pub method: String,
    pub params: CallFunctionParameters,
}

impl CallFunction {
    pub fn new(params: CallFunctionParameters) -> Self {
        Self {
            method: "script.callFunction".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallFunctionParameters {
    #[serde(rename = "functionDeclaration")]
    pub function_declaration: String,
    #[serde(rename = "awaitPromise")]
    pub await_promise: bool,
    pub target: Target,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<LocalValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "resultOwnership")]
    pub result_ownership: Option<ResultOwnership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "serializationOptions")]
    pub serialization_options: Option<SerializationOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub this: Option<LocalValue>,
    #[serde(rename = "userActivation", skip_serializing_if = "Option::is_none")]
    pub user_activation: Option<bool>,
}

impl CallFunctionParameters {
    pub fn new(
        function_declaration: String,
        await_promise: bool,
        target: Target,
        arguments: Option<Vec<LocalValue>>,
        result_ownership: Option<ResultOwnership>,
        serialization_options: Option<SerializationOptions>,
        this: Option<LocalValue>,
        user_activation: Option<bool>,
    ) -> Self {
        Self {
            function_declaration,
            await_promise,
            target,
            arguments,
            result_ownership,
            serialization_options,
            this,
            user_activation,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Evaluate {
    pub method: String,
    pub params: EvaluateParameters,
}

impl Evaluate {
    pub fn new(params: EvaluateParameters) -> Self {
        Self {
            method: "script.evaluate".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluateParameters {
    pub expression: String,
    pub target: Target,
    #[serde(rename = "awaitPromise")]
    pub await_promise: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "resultOwnership")]
    pub result_ownership: Option<ResultOwnership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "serializationOptions")]
    pub serialization_options: Option<SerializationOptions>,
    #[serde(rename = "userActivation", skip_serializing_if = "Option::is_none")]
    pub user_activation: Option<bool>,
}

impl EvaluateParameters {
    pub fn new(
        expression: String,
        target: Target,
        await_promise: bool,
        result_ownership: Option<ResultOwnership>,
        serialization_options: Option<SerializationOptions>,
        user_activation: Option<bool>,
    ) -> Self {
        Self {
            expression,
            target,
            await_promise,
            result_ownership,
            serialization_options,
            user_activation,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRealms {
    pub method: String,
    pub params: GetRealmsParameters,
}

impl GetRealms {
    pub fn new(params: GetRealmsParameters) -> Self {
        Self {
            method: "script.getRealms".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetRealmsParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<BrowsingContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub realm_type: Option<RealmType>,
}

impl GetRealmsParameters {
    pub fn new(context: Option<BrowsingContext>, realm_type: Option<RealmType>) -> Self {
        Self {
            context,
            realm_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRealmsResult {
    pub realms: Vec<RealmInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemovePreloadScript {
    pub method: String,
    pub params: RemovePreloadScriptParameters,
}

impl RemovePreloadScript {
    pub fn new(params: RemovePreloadScriptParameters) -> Self {
        Self {
            method: "script.removePreloadScript".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemovePreloadScriptParameters {
    pub script: PreloadScript,
}

impl RemovePreloadScriptParameters {
    pub fn new(script: PreloadScript) -> Self {
        Self { script }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub method: String,
    pub params: MessageParameters,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageParameters {
    pub channel: Channel,
    pub data: RemoteValue,
    pub source: Source,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RealmCreated {
    pub method: String,
    pub params: RealmInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RealmDestroyed {
    pub method: String,
    pub params: RealmDestroyedParameters,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RealmDestroyedParameters {
    pub realm: Realm,
}
