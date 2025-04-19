pub mod webdriver {
    pub mod capabilities;
    pub mod session;
}
pub mod session;
pub mod model {
    pub mod browser;
    pub mod browsing_context;
    pub mod command;
    pub mod common;
    pub mod emulation;
    pub mod error;
    pub mod event;
    pub mod input;
    pub mod log;
    pub mod message;
    pub mod network;
    pub mod result;
    pub mod script;
    pub mod session;
    pub mod storage;
    pub mod web_extension;
}
mod commands {
    pub mod browser;
    pub mod browsing_context;
    mod id;
    pub mod session;
    #[macro_use]
    mod utils;
    pub mod emulation;
    pub mod input;
    pub mod network;
    pub mod script;
    pub mod storage;
    pub mod web_extension;
}
mod command_sender;
pub mod error;
pub mod events;
mod message_handler;
