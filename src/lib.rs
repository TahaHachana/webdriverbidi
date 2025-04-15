pub mod webdriver {
    pub mod capabilities;
    pub mod session;
}
pub mod session;
//pub mod model;
pub mod models {
    pub mod local;
    pub mod remote;
}
mod commands {
    pub mod browser;
    pub mod browsing_context;
    mod id;
    pub mod session;
    #[macro_use]
    mod utils;
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

// Re-export key structs and enums
pub use models::local;
pub use models::remote;
