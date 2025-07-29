use serde::{Deserialize, Serialize};

use super::id;
use super::utils;

use crate::error::CommandError;
use crate::model::network::*;
use crate::model::result::EmptyResult;
use crate::session::WebDriverBiDiSession;

// https://w3c.github.io/webdriver-bidi/#command-network-addDataCollector
define_command!(
    AddDataCollectorCommand,
    AddDataCollector,
    AddDataCollectorParameters,
    add_data_collector,
    AddDataCollectorResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-addIntercept
define_command!(
    AddInterceptCommand,
    AddIntercept,
    AddInterceptParameters,
    add_intercept,
    AddInterceptResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-continueRequest
define_command!(
    ContinueRequestCommand,
    ContinueRequest,
    ContinueRequestParameters,
    continue_request,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-continueResponse
define_command!(
    ContinueResponseCommand,
    ContinueResponse,
    ContinueResponseParameters,
    continue_response,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-continueWithAuth
define_command!(
    ContinueWithAuthCommand,
    ContinueWithAuth,
    ContinueWithAuthParameters,
    continue_with_auth,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-disownData
define_command!(
    DisownDataCommand,
    DisownData,
    DisownDataParameters,
    disown_data,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-failRequest
define_command!(
    FailRequestCommand,
    FailRequest,
    FailRequestParameters,
    fail_request,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-getData
define_command!(
    GetDataCommand,
    GetData,
    GetDataParameters,
    get_data,
    GetDataResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-provideResponse
define_command!(
    ProvideResponseCommand,
    ProvideResponse,
    ProvideResponseParameters,
    provide_response,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-removeDataCollector
define_command!(
    RemoveDataCollectorCommand,
    RemoveDataCollector,
    RemoveDataCollectorParameters,
    remove_data_collector,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-removeIntercept
define_command!(
    RemoveInterceptCommand,
    RemoveIntercept,
    RemoveInterceptParameters,
    remove_intercept,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-setCacheBehavior
define_command!(
    SetCacheBehaviorCommand,
    SetCacheBehavior,
    SetCacheBehaviorParameters,
    set_cache_behavior,
    EmptyResult
);

// https://w3c.github.io/webdriver-bidi/#command-network-setExtraHeaders
define_command!(
    SetExtraHeadersCommand,
    SetExtraHeaders,
    SetExtraHeadersParameters,
    set_extra_headers,
    EmptyResult
);
