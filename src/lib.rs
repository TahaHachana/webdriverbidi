pub mod webdriver {
    pub mod capabilities;
    pub mod session;
}
pub mod session;
pub mod models {
    pub mod local;
    pub mod remote;
}
mod commands {
    pub mod browser;
    pub mod browsing_context;
    mod id;
    pub mod session;
    mod utils;
    pub mod network;
}
mod command_sender;
mod error;
pub mod events;
mod message_handler;

// Re-export key functionality
pub use models::local;
pub use models::remote;
