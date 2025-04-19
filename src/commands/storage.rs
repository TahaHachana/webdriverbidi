use serde::{Deserialize, Serialize};

use super::id;
use super::utils;

use crate::define_command;
use crate::error::CommandError;
use crate::model::storage::*;
use crate::session::WebDriverBiDiSession;

// https://w3c.github.io/webdriver-bidi/#command-storage-getCookies
define_command!(
    GetCookiesCommand,
    GetCookies,
    GetCookiesParameters,
    get_cookies,
    GetCookiesResult
);

// https://w3c.github.io/webdriver-bidi/#command-storage-setCookie
define_command!(
    SetCookieCommand,
    SetCookie,
    SetCookieParameters,
    set_cookie,
    SetCookieResult
);

// https://w3c.github.io/webdriver-bidi/#command-storage-deleteCookies
define_command!(
    DeleteCookiesCommand,
    DeleteCookies,
    DeleteCookiesParameters,
    delete_cookies,
    DeleteCookiesResult
);
