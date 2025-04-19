use serde::{Deserialize, Serialize};

use super::id;
use super::utils;

use crate::define_command;
use crate::error::CommandError;
use crate::model::result::EmptyResult;
use crate::model::script::*;
use crate::session::WebDriverBiDiSession;

// https://w3c.github.io/webdriver-bidi/#command-script-addPreloadScript
define_command!(
    AddPreloadScriptCommand,
    AddPreloadScript,
    AddPreloadScriptParameters,
    add_preload_script,
    AddPreloadScriptResult
);

// https://w3c.github.io/webdriver-bidi/#command-script-disown
define_command!(DisownCommand, Disown, DisownParameters, disown, EmptyResult);

// https://w3c.github.io/webdriver-bidi/#command-script-callFunction
define_command!(
    CallFunctionCommand,
    CallFunction,
    CallFunctionParameters,
    call_function,
    EvaluateResult
);

// https://w3c.github.io/webdriver-bidi/#command-script-evaluate
define_command!(
    EvaluateCommand,
    Evaluate,
    EvaluateParameters,
    evaluate,
    EvaluateResult
);

// https://w3c.github.io/webdriver-bidi/#command-script-getRealms
define_command!(
    GetRealmsCommand,
    GetRealms,
    GetRealmsParameters,
    get_realms,
    GetRealmsResult
);

// https://w3c.github.io/webdriver-bidi/#command-script-removePreloadScript
define_command!(
    RemovePreloadScriptCommand,
    RemovePreloadScript,
    RemovePreloadScriptParameters,
    remove_preload_script,
    EmptyResult
);
