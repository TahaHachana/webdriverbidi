use serde::{Deserialize, Serialize};

use crate::model::browsing_context::{BrowsingContext, Navigation};
use crate::model::common::{Extensible, JsInt, JsUint};
use crate::model::script::StackTrace;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NetworkCommand {
    AddIntercept(AddIntercept),
    ContinueRequest(ContinueRequest),
    ContinueResponse(ContinueResponse),
    ContinueWithAuth(ContinueWithAuth),
    FailRequest(FailRequest),
    ProvideResponse(ProvideResponse),
    RemoveIntercept(RemoveIntercept),
    SetCacheBehavior(SetCacheBehavior),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkResult {
    AddInterceptResult(AddInterceptResult),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum NetworkEvent {
    AuthRequired(AuthRequired),
    BeforeRequestSent(BeforeRequestSent),
    FetchError(FetchError),
    ResponseCompleted(ResponseCompleted),
    ResponseStarted(ResponseStarted),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthChallenge {
    pub scheme: String,
    pub realm: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthCredentials {
    #[serde(rename = "type")]
    pub auth_credentials_type: String,
    pub username: String,
    pub password: String,
}

impl AuthCredentials {
    pub fn new(username: String, password: String) -> Self {
        Self {
            auth_credentials_type: "password".to_string(),
            username,
            password,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BaseParameters {
    pub context: Option<BrowsingContext>,
    pub is_blocked: bool,
    pub navigation: Option<Navigation>,
    pub redirect_count: JsUint,
    pub request: RequestData,
    pub timestamp: JsUint,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intercepts: Option<Vec<Intercept>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BytesValue {
    StringValue(StringValue),
    Base64Value(Base64Value),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StringValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Base64Value {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cookie {
    pub name: String,
    pub value: BytesValue,
    pub domain: String,
    pub path: String,
    pub size: JsUint,
    #[serde(rename = "httpOnly")]
    pub http_only: bool,
    pub secure: bool,
    #[serde(rename = "sameSite")]
    pub same_site: SameSite,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<JsUint>,
    #[serde(flatten)]
    pub extensible: Extensible,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CookieHeader {
    pub name: String,
    pub value: BytesValue,
}

impl CookieHeader {
    pub fn new(name: String, value: BytesValue) -> Self {
        Self { name, value }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchTimingInfo {
    #[serde(rename = "timeOrigin")]
    pub time_origin: f64,
    #[serde(rename = "requestTime")]
    pub request_time: f64,
    #[serde(rename = "redirectStart")]
    pub redirect_start: f64,
    #[serde(rename = "redirectEnd")]
    pub redirect_end: f64,
    #[serde(rename = "fetchStart")]
    pub fetch_start: f64,
    #[serde(rename = "dnsStart")]
    pub dns_start: f64,
    #[serde(rename = "dnsEnd")]
    pub dns_end: f64,
    #[serde(rename = "connectStart")]
    pub connect_start: f64,
    #[serde(rename = "connectEnd")]
    pub connect_end: f64,
    #[serde(rename = "tlsStart")]
    pub tls_start: f64,
    #[serde(rename = "requestStart")]
    pub request_start: f64,
    #[serde(rename = "responseStart")]
    pub response_start: f64,
    #[serde(rename = "responseEnd")]
    pub response_end: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: BytesValue,
}

impl Header {
    pub fn new(name: String, value: BytesValue) -> Self {
        Self { name, value }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Initiator {
    #[serde(rename = "columnNumber", skip_serializing_if = "Option::is_none")]
    pub column_number: Option<JsUint>,
    #[serde(rename = "lineNumber", skip_serializing_if = "Option::is_none")]
    pub line_number: Option<JsUint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<Request>,
    #[serde(rename = "stackTrace", skip_serializing_if = "Option::is_none")]
    pub stack_trace: Option<StackTrace>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub initiator_type: Option<String>,
}

pub type Intercept = String;
pub type Request = String;

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestData {
    pub request: Request,
    pub url: String,
    pub method: String,
    pub headers: Vec<Header>,
    pub cookies: Vec<Cookie>,
    #[serde(rename = "headersSize")]
    pub headers_size: JsUint,
    #[serde(rename = "bodySize")]
    pub body_size: Option<JsUint>,
    pub destination: String,
    #[serde(rename = "initiatorType")]
    pub initiator_type: Option<String>,
    pub timings: FetchTimingInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseContent {
    pub size: JsUint,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseData {
    pub url: String,
    pub protocol: String,
    pub status: JsUint,
    #[serde(rename = "statusText")]
    pub status_text: String,
    #[serde(rename = "fromCache")]
    pub from_cache: bool,
    pub headers: Vec<Header>,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(rename = "bytesReceived")]
    pub bytes_received: JsUint,
    #[serde(rename = "headersSize")]
    pub headers_size: Option<JsUint>,
    #[serde(rename = "bodySize")]
    pub body_size: Option<JsUint>,
    pub content: ResponseContent,
    #[serde(rename = "authChallenges", skip_serializing_if = "Option::is_none")]
    pub auth_challenges: Option<Vec<AuthChallenge>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetCookieHeader {
    pub name: String,
    pub value: BytesValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(rename = "httpOnly", skip_serializing_if = "Option::is_none")]
    pub http_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(rename = "maxAge", skip_serializing_if = "Option::is_none")]
    pub max_age: Option<JsInt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(rename = "sameSite", skip_serializing_if = "Option::is_none")]
    pub same_site: Option<SameSite>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secure: Option<bool>,
}

impl SetCookieHeader {
    pub fn new(
        name: String,
        value: BytesValue,
        domain: Option<String>,
        http_only: Option<bool>,
        expiry: Option<String>,
        max_age: Option<JsInt>,
        path: Option<String>,
        same_site: Option<SameSite>,
        secure: Option<bool>,
    ) -> Self {
        Self {
            name,
            value,
            domain,
            http_only,
            expiry,
            max_age,
            path,
            same_site,
            secure,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UrlPattern {
    UrlPatternPattern(UrlPatternPattern),
    UrlPatternString(UrlPatternString),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlPatternPattern {
    #[serde(rename = "type")]
    pub url_pattern_pattern_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pathname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
}

impl UrlPatternPattern {
    pub fn new(
        protocol: Option<String>,
        hostname: Option<String>,
        port: Option<String>,
        pathname: Option<String>,
        search: Option<String>,
    ) -> Self {
        Self {
            url_pattern_pattern_type: "pattern".to_string(),
            protocol,
            hostname,
            port,
            pathname,
            search,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UrlPatternString {
    #[serde(rename = "type")]
    pub url_pattern_string_type: String,
    pub pattern: String,
}

impl UrlPatternString {
    pub fn new(pattern: String) -> Self {
        Self {
            url_pattern_string_type: "string".to_string(),
            pattern,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddIntercept {
    pub method: String,
    pub params: AddInterceptParameters,
}

impl AddIntercept {
    pub fn new(params: AddInterceptParameters) -> Self {
        Self {
            method: "network.addIntercept".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddInterceptParameters {
    pub phases: Vec<InterceptPhase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_patterns: Option<Vec<UrlPattern>>,
}

impl AddInterceptParameters {
    pub fn new(
        phases: Vec<InterceptPhase>,
        contexts: Option<Vec<BrowsingContext>>,
        url_patterns: Option<Vec<UrlPattern>>,
    ) -> Self {
        Self {
            phases,
            contexts,
            url_patterns,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InterceptPhase {
    #[serde(rename = "beforeRequestSent")]
    BeforeRequestSent,
    #[serde(rename = "responseStarted")]
    ResponseStarted,
    #[serde(rename = "authRequired")]
    AuthRequired,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddInterceptResult {
    pub intercept: Intercept,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContinueRequest {
    pub method: String,
    pub params: ContinueRequestParameters,
}

impl ContinueRequest {
    pub fn new(params: ContinueRequestParameters) -> Self {
        Self {
            method: "network.continueRequest".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContinueRequestParameters {
    pub request: Request,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<BytesValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<Vec<CookieHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<Header>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl ContinueRequestParameters {
    pub fn new(
        request: Request,
        body: Option<BytesValue>,
        cookies: Option<Vec<CookieHeader>>,
        headers: Option<Vec<Header>>,
        method: Option<String>,
        url: Option<String>,
    ) -> Self {
        Self {
            request,
            body,
            cookies,
            headers,
            method,
            url,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContinueResponse {
    pub method: String,
    pub params: ContinueResponseParameters,
}

impl ContinueResponse {
    pub fn new(params: ContinueResponseParameters) -> Self {
        Self {
            method: "network.continueResponse".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContinueResponseParameters {
    pub request: Request,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<Vec<SetCookieHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<AuthCredentials>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<Header>>,
    #[serde(rename = "reasonPhrase", skip_serializing_if = "Option::is_none")]
    pub reason_phrase: Option<String>,
    #[serde(rename = "statusCode", skip_serializing_if = "Option::is_none")]
    pub status_code: Option<JsUint>,
}

impl ContinueResponseParameters {
    pub fn new(
        request: Request,
        cookies: Option<Vec<SetCookieHeader>>,
        credentials: Option<AuthCredentials>,
        headers: Option<Vec<Header>>,
        reason_phrase: Option<String>,
        status_code: Option<JsUint>,
    ) -> Self {
        Self {
            request,
            cookies,
            credentials,
            headers,
            reason_phrase,
            status_code,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContinueWithAuth {
    pub method: String,
    pub params: ContinueWithAuthParameters,
}

impl ContinueWithAuth {
    pub fn new(params: ContinueWithAuthParameters) -> Self {
        Self {
            method: "network.continueWithAuth".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContinueWithAuthParameters {
    pub request: Request,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub auth_option: Option<ContinueWithAuthOption>,
}

impl ContinueWithAuthParameters {
    pub fn new(request: Request, auth_option: Option<ContinueWithAuthOption>) -> Self {
        Self {
            request,
            auth_option,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContinueWithAuthOption {
    Credentials(ContinueWithAuthCredentials),
    NoCredentials(ContinueWithAuthNoCredentials),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContinueWithAuthCredentials {
    pub action: String,
    pub credentials: AuthCredentials,
}

impl ContinueWithAuthCredentials {
    pub fn new(action: String, credentials: AuthCredentials) -> Self {
        Self {
            action,
            credentials,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContinueWithAuthNoCredentials {
    pub action: NoCredentialsAction,
}

impl ContinueWithAuthNoCredentials {
    pub fn new(action: NoCredentialsAction) -> Self {
        Self { action }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoCredentialsAction {
    Default,
    Cancel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailRequest {
    pub method: String,
    pub params: FailRequestParameters,
}

impl FailRequest {
    pub fn new(params: FailRequestParameters) -> Self {
        Self {
            method: "network.failRequest".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailRequestParameters {
    pub request: Request,
}

impl FailRequestParameters {
    pub fn new(request: Request) -> Self {
        Self { request }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvideResponse {
    pub method: String,
    pub params: ProvideResponseParameters,
}

impl ProvideResponse {
    pub fn new(params: ProvideResponseParameters) -> Self {
        Self {
            method: "network.provideResponse".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvideResponseParameters {
    pub request: Request,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<BytesValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<Vec<SetCookieHeader>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<Header>>,
    #[serde(rename = "reasonPhrase", skip_serializing_if = "Option::is_none")]
    pub reason_phrase: Option<String>,
    #[serde(rename = "statusCode", skip_serializing_if = "Option::is_none")]
    pub status_code: Option<JsUint>,
}

impl ProvideResponseParameters {
    pub fn new(
        request: Request,
        body: Option<BytesValue>,
        cookies: Option<Vec<SetCookieHeader>>,
        headers: Option<Vec<Header>>,
        reason_phrase: Option<String>,
        status_code: Option<JsUint>,
    ) -> Self {
        Self {
            request,
            body,
            cookies,
            headers,
            reason_phrase,
            status_code,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveIntercept {
    pub method: String,
    pub params: RemoveInterceptParameters,
}

impl RemoveIntercept {
    pub fn new(params: RemoveInterceptParameters) -> Self {
        Self {
            method: "network.removeIntercept".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveInterceptParameters {
    pub intercept: Intercept,
}

impl RemoveInterceptParameters {
    pub fn new(intercept: Intercept) -> Self {
        Self { intercept }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetCacheBehavior {
    pub method: String,
    pub params: SetCacheBehaviorParameters,
}

impl SetCacheBehavior {
    pub fn new(params: SetCacheBehaviorParameters) -> Self {
        Self {
            method: "network.setCacheBehavior".to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetCacheBehaviorParameters {
    pub cache_behavior: CacheBehavior,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Vec<BrowsingContext>>,
}

impl SetCacheBehaviorParameters {
    pub fn new(cache_behavior: CacheBehavior, contexts: Option<Vec<BrowsingContext>>) -> Self {
        Self {
            cache_behavior,
            contexts,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheBehavior {
    Default,
    Bypass,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequired {
    pub method: String,
    pub params: AuthRequiredParameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequiredParameters {
    #[serde(flatten)]
    pub base: BaseParameters,
    pub response: ResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BeforeRequestSent {
    pub method: String,
    pub params: BeforeRequestSentParameters,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BeforeRequestSentParameters {
    #[serde(flatten)]
    pub base: BaseParameters,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiator: Option<Initiator>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchError {
    pub method: String,
    pub params: FetchErrorParameters,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchErrorParameters {
    #[serde(flatten)]
    pub base: BaseParameters,
    #[serde(rename = "errorText")]
    pub error_text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseCompleted {
    pub method: String,
    pub params: ResponseCompletedParameters,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseCompletedParameters {
    #[serde(flatten)]
    pub base: BaseParameters,
    pub response: ResponseData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseStarted {
    pub method: String,
    pub params: ResponseStartedParameters,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseStartedParameters {
    #[serde(flatten)]
    pub base: BaseParameters,
    pub response: ResponseData,
}
