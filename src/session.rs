use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use log::debug;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, oneshot};
use tokio::task;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::command_sender;
use crate::commands;
use crate::error::{CommandError, SessionError};
use crate::events::EventType;
use crate::message_handler;
use crate::model::browser::ClientWindowInfo;
use crate::model::browser::*;
use crate::model::browsing_context::*;
use crate::model::common::EmptyParams;
use crate::model::emulation::*;
use crate::model::input::*;
use crate::model::network::*;
use crate::model::result::EmptyResult;
use crate::model::script::EvaluateResult;
use crate::model::script::*;
use crate::model::session::*;
use crate::model::storage::*;
use crate::model::web_extension::*;
use crate::webdriver::capabilities::CapabilitiesRequest;
use crate::webdriver::session;

/// Type alias for the event handler functions.
pub type EventHandler =
    Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Represents a WebDriver BiDi session.
///
/// This struct manages the lifecycle of a WebDriver session, including
/// starting the session, establishing a WebSocket connection, sending
/// commands, handling incoming messages whether they are command responses
/// or events and eventually closing the session.
///
/// # Fields
///
/// * `host` - The host address of the WebDriver server.
/// * `port` - The port number of the WebDriver server.
/// * `base_url` - The base URL constructed from the host and port.
/// * `session_id` - The unique identifier for the session.
/// * `capabilities` - The desired capabilities for the session.
/// * `websocket_url` - The WebSocket URL for bidirectional communication.
/// * `websocket_stream` - The WebSocket stream for communication protected by an `Arc` wrapped `Mutex`.
/// * `pending_commands` - A map of pending commands awaiting responses protected by an `Arc` wrapped `Mutex`.
/// * `event_handlers` - A map of events and their handlers protected by an `Arc` wrapped `Mutex`.
#[derive(Clone)]
pub struct WebDriverBiDiSession {
    pub host: String,
    pub port: u16,
    pub base_url: String,
    pub session_id: String,
    pub capabilities: CapabilitiesRequest,
    pub websocket_url: String,
    pub websocket_stream: Option<Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    pub pending_commands: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    event_handlers: Arc<Mutex<HashMap<EventType, EventHandler>>>,
}

impl WebDriverBiDiSession {
    /// Creates a new session.
    ///
    /// # Arguments
    ///
    /// * `host` - The host address of the WebDriver server.
    /// * `port` - The port number of the WebDriver server.
    /// * `capabilities` - The desired capabilities for the session.
    pub fn new(host: String, port: u16, capabilities: CapabilitiesRequest) -> Self {
        let base_url = format!("http://{}:{}", host, port);
        debug!("Constructed base URL: {}", base_url);
        Self {
            host,
            port,
            base_url,
            session_id: String::new(),
            capabilities,
            websocket_url: String::new(),
            websocket_stream: None,
            pending_commands: Arc::new(Mutex::new(HashMap::new())),
            event_handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start a WebDriver session, establishe a WebSocket connection and
    /// spawn a background task to handle incoming messages.
    ///
    /// **A WebDriver BiDi server must be running before calling this method.**
    pub async fn start(&mut self) -> Result<(), SessionError> {
        let session = session::start_session(&self.base_url, &self.capabilities)
            .await
            .map_err(|e| SessionError::Other(format!("Failed to start session: {}", e)))?;
        self.session_id = session.session_id;
        self.websocket_url = session.websocket_url;

        debug!("Establishing the WebSocket connection");
        let (stream, _) = connect_async(&self.websocket_url)
            .await
            .map_err(|e| SessionError::Other(format!("Failed to connect to WebSocket: {}", e)))?;

        let websocket_stream = Arc::new(Mutex::new(stream));
        self.websocket_stream = Some(websocket_stream.clone());

        let pending_commands = self.pending_commands.clone();
        let event_handlers = self.event_handlers.clone();

        debug!("Starting the incoming messages management loop");
        // Spawn a background task to manage incoming messages
        self.spawn_message_handler_task(websocket_stream, pending_commands, event_handlers);

        Ok(())
    }

    /// Close the WebDriver session.
    pub async fn close(&mut self) -> Result<(), SessionError> {
        session::close_session(&self.base_url, &self.session_id).await?;
        Ok(())
    }

    /// Send a WebDriver BiDi command.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to send.
    ///
    /// # Returns
    ///
    /// A result containing the response of type `U` that implements the `DeserializeOwned` trait,
    /// or a `CommandError` if the command could not be sent.
    pub async fn send_command<T: Serialize, U: DeserializeOwned>(
        &mut self,
        command: T,
    ) -> Result<U, CommandError> {
        if let Some(websocket_stream) = &self.websocket_stream {
            command_sender::send_command(
                websocket_stream.clone(),
                self.pending_commands.clone(),
                command,
            )
            .await
        } else {
            let error_msg = "WebSocket stream not initialized.";
            Err(CommandError::Other(error_msg.into()))
        }
    }

    /// Spawn a background task to manage incoming WebSocket messages.
    ///
    /// This method creates a new asynchronous task that continuously listens for
    /// incoming messages on the WebSocket connection and handles them appropriately.
    fn spawn_message_handler_task(
        &self,
        websocket_stream: Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
        pending_commands: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
        event_handlers: Arc<Mutex<HashMap<EventType, EventHandler>>>,
    ) {
        task::spawn(message_handler::handle_messages(
            websocket_stream,
            pending_commands,
            event_handlers,
        ));
    }

    /// Register an event handler for a specific event type.
    ///
    /// # Arguments
    ///
    /// * `event_type` - The type of the event to handle.
    /// * `handler` - The event handler function.
    pub async fn register_event_handler<F, Fut>(&mut self, event_type: EventType, handler: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        debug!("Registring event handler for event: {:?}", event_type);
        let mut handlers = self.event_handlers.lock().await;
        handlers.insert(event_type, Box::new(move |event| Box::pin(handler(event))));
    }

    /// Unregister an event handler for a specific event type.
    ///
    /// # Arguments
    ///
    /// * `event_type` - The type of the event to stop handling.
    pub async fn unregister_event_handler(&mut self, event_type: EventType) {
        let mut handlers = self.event_handlers.lock().await;
        handlers.remove(&event_type);
    }
}

// Browsing context commands
impl WebDriverBiDiSession {
    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-activate

    /// Activate and focus a browsing context.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `ActivateParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn browsing_context_activate(
        &mut self,
        params: ActivateParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::browsing_context::activate(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-captureScreenshot

    /// Capture an image of the given navigable and return it as a Base64-encoded string.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `CaptureScreenshotParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `CaptureScreenshotResult` or a `CommandError`.
    pub async fn browsing_context_capture_screenshot(
        &mut self,
        params: CaptureScreenshotParameters,
    ) -> Result<CaptureScreenshotResult, CommandError> {
        commands::browsing_context::capture_screenshot(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-close

    /// Close the browsing context.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `CloseParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn browsing_context_close(
        &mut self,
        params: CloseParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::browsing_context::close(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-create

    /// Create a new browsing context.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `CreateParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `CreateResult` or a `CommandError`.
    pub async fn browsing_context_create(
        &mut self,
        params: CreateParameters,
    ) -> Result<CreateResult, CommandError> {
        commands::browsing_context::create(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-getTree

    /// Retrieve the browsing context tree.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a GetTreeParameters instance.
    ///
    /// # Returns
    ///
    /// A result containing the `GetTreeResult` or a `CommandError`.
    pub async fn browsing_context_get_tree(
        &mut self,
        params: GetTreeParameters,
    ) -> Result<GetTreeResult, CommandError> {
        commands::browsing_context::get_tree(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-handleUserPrompt

    /// Allow closing an open prompt.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `HandleUserPromptParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn browsing_context_handle_user_prompt(
        &mut self,
        params: HandleUserPromptParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::browsing_context::handle_user_prompt(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-locateNodes

    /// Return a list of all nodes matching the specified locator.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `LocateNodesParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `LocateNodesResult` or a `CommandError`.
    pub async fn browsing_context_locate_nodes(
        &mut self,
        params: LocateNodesParameters,
    ) -> Result<LocateNodesResult, CommandError> {
        commands::browsing_context::locate_nodes(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-navigate

    /// Navigate to a URL in the browsing context.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a NavigateParameters instance.
    ///
    /// # Returns
    ///
    /// A result containing the `NavigateResult` or a `CommandError`.
    pub async fn browsing_context_navigate(
        &mut self,
        params: NavigateParameters,
    ) -> Result<NavigateResult, CommandError> {
        commands::browsing_context::navigate(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-print

    /// Create a paginated representation of a document and return it
    /// as a Base64-encoded string PDF.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `PrintParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `PrintResult` or a `CommandError`.
    pub async fn browsing_context_print(
        &mut self,
        params: PrintParameters,
    ) -> Result<PrintResult, CommandError> {
        commands::browsing_context::print(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-reload

    /// Reload the current page in the browsing context.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `ReloadParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `NavigateResult` or a `CommandError`.
    pub async fn browsing_context_reload(
        &mut self,
        params: ReloadParameters,
    ) -> Result<NavigateResult, CommandError> {
        commands::browsing_context::reload(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-setViewport

    /// Modify specific viewport characteristics (e.g. viewport width and viewport
    /// height) on the given top-level traversable.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `SetViewportParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn browsing_context_set_viewport(
        &mut self,
        params: SetViewportParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::browsing_context::set_viewport(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browsingContext-traverseHistory

    /// Navigate through the browsing history of a specified context.
    ///
    /// This method allows you to move forward or backward in the browsing history
    /// of a given navigable context by a specified number of steps (delta).
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `TraverseHistoryParameters` instance, which
    ///   includes the context identifier and the delta indicating the number of steps
    ///   to move in the history. A positive delta moves forward, while a negative delta
    ///   moves backward.
    ///
    /// # Returns
    ///
    /// A result containing the `TraverseHistoryResult` or a `CommandError` if the
    /// operation fails.
    pub async fn browsing_context_traverse_history(
        &mut self,
        params: TraverseHistoryParameters,
    ) -> Result<TraverseHistoryResult, CommandError> {
        commands::browsing_context::traverse_history(self, params).await
    }
}

// Session commands
impl WebDriverBiDiSession {
    // https://w3c.github.io/webdriver-bidi/#command-session-status

    /// Return information about whether a remote end is in a state
    /// in which it can create new sessions, but may additionally include
    /// arbitrary meta information that is specific to the implementation.
    ///
    /// # Returns
    ///
    /// A result containing the `SessionStatus` or a `CommandError`.
    pub async fn session_status(
        &mut self,
        params: EmptyParams,
    ) -> Result<StatusResult, CommandError> {
        commands::session::status(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-session-new

    /// Create a new session.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `NewParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `NewResult` or a `CommandError`.
    pub async fn session_new(&mut self, params: NewParameters) -> Result<NewResult, CommandError> {
        commands::session::new(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-session-end

    /// End the current session.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `EmptyParams` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn session_end(&mut self, params: EmptyParams) -> Result<EmptyResult, CommandError> {
        commands::session::end(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-session-subscribe

    /// Enable certain events either globally or for a set of navigables.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `SubscriptionRequest` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `SubscriptionRequestResult` or a `CommandError`.
    pub async fn session_subscribe(
        &mut self,
        params: SubscriptionRequest,
    ) -> Result<SubscribeResult, CommandError> {
        commands::session::subscribe(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-session-unsubscribe

    /// Disable certain events either globally or for a set of navigables.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `UnsubscribeRequest` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn session_unsubscribe(
        &mut self,
        params: UnsubscribeParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::session::unsubscribe(self, params).await
    }
}

// Browser commands
impl WebDriverBiDiSession {
    // https://w3c.github.io/webdriver-bidi/#command-browser-close

    /// Terminate all WebDriver sessions and clean up automation state
    /// in the remote browser instance.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `EmptyParams` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn browser_close(
        &mut self,
        params: EmptyParams,
    ) -> Result<EmptyResult, CommandError> {
        commands::browser::close(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browser-createUserContext

    /// Create a new user context.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `EmptyParams` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `CreateUserContextResult` or a `CommandError`.
    pub async fn browser_create_user_context(
        &mut self,
        params: CreateUserContextParameters,
    ) -> Result<CreateUserContextResult, CommandError> {
        commands::browser::create_user_context(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browser-getClientWindows

    /// Retrieve the list of client windows.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `EmptyParams` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `GetClientWindowsResult` or a `CommandError`.
    pub async fn browser_get_client_windows(
        &mut self,
        params: EmptyParams,
    ) -> Result<GetClientWindowsResult, CommandError> {
        commands::browser::get_client_windows(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browser-getUserContexts

    /// Retrieve the list of user contexts.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `EmptyParams` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `GetUserContextsResult` or a `CommandError`.
    pub async fn browser_get_user_contexts(
        &mut self,
        params: EmptyParams,
    ) -> Result<GetUserContextsResult, CommandError> {
        commands::browser::get_user_contexts(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browser-removeUserContext

    /// Close a user context and all navigables in it.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `RemoveUserContextParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn browser_remove_user_context(
        &mut self,
        params: RemoveUserContextParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::browser::remove_user_context(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-browser-setClientWindowState

    /// Set the dimensions of a client window.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `SetClientWindowStateParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `ClientWindowInfo` or a `CommandError`.
    pub async fn browser_set_client_window_state(
        &mut self,
        params: SetClientWindowStateParameters,
    ) -> Result<ClientWindowInfo, CommandError> {
        commands::browser::set_client_window_state(self, params).await
    }
}

// Emulation commands
impl WebDriverBiDiSession {
    // https://w3c.github.io/webdriver-bidi/#command-emulation-setGeolocationOverride

    /// Modify geolocation characteristics on the given top-level traversables or user contexts.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `SetGeolocationOverrideParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn set_geolocation_override(
        &mut self,
        params: SetGeolocationOverrideParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::emulation::set_geolocation_override(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-emulation-setLocaleOverride

    /// Modifies locale on the given top-level traversables or user contexts.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `SetLocaleOverrideParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn set_locale_override(
        &mut self,
        params: SetLocaleOverrideParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::emulation::set_locale_override(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-emulation-setScreenOrientationOverride

    /// Emulates screen orientation of the given top-level traversables or user contexts.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `SetScreenOrientationOverrideParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn set_screen_orientation_override(
        &mut self,
        params: SetScreenOrientationOverrideParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::emulation::set_screen_orientation_override(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-emulation-setTimezoneOverride

    /// Modifies timezone on the given top-level traversables or user contexts.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `SetTimezoneOverrideParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn set_timezone_override(
        &mut self,
        params: SetTimezoneOverrideParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::emulation::set_timezone_override(self, params).await
    }
}

// Network commands
impl WebDriverBiDiSession {
    // https://w3c.github.io/webdriver-bidi/#command-network-addIntercept

    /// Add a network intercept.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `AddInterceptParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `AddInterceptResult` or a `CommandError`.
    pub async fn network_add_intercept(
        &mut self,
        params: AddInterceptParameters,
    ) -> Result<AddInterceptResult, CommandError> {
        commands::network::add_intercept(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-network-continueRequest

    /// Continue a request that’s blocked by a network intercept.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `ContinueRequestParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn network_continue_request(
        &mut self,
        params: ContinueRequestParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::network::continue_request(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-network-continueResponse

    /// Continue a response that’s blocked by a network intercept.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `ContinueResponseParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn network_continue_response(
        &mut self,
        params: ContinueResponseParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::network::continue_response(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-network-continueWithAuth

    /// Continue a request that’s blocked by a network intercept at the authRequired phase.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `ContinueWithAuthParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn network_continue_with_auth(
        &mut self,
        params: ContinueWithAuthParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::network::continue_with_auth(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-network-failRequest

    /// Fail a fetch that’s blocked by a network intercept.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `FailRequestParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn network_fail_request(
        &mut self,
        params: FailRequestParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::network::fail_request(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-network-provideResponse

    /// Continue a request that’s blocked by a network intercept, by providing a complete response.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `ProvideResponseParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn network_provide_response(
        &mut self,
        params: ProvideResponseParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::network::provide_response(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-network-removeIntercept

    /// Remove a network intercept.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `RemoveInterceptParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn network_remove_intercept(
        &mut self,
        params: RemoveInterceptParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::network::remove_intercept(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-network-setCacheBehavior

    /// Configure the network cache behavior for certain requests.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `SetCacheBehaviorParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn network_set_cache_behavior(
        &mut self,
        params: SetCacheBehaviorParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::network::set_cache_behavior(self, params).await
    }
}

// Script commands
impl WebDriverBiDiSession {
    // https://w3c.github.io/webdriver-bidi/#command-script-addPreloadScript

    /// Add a script to be preloaded into the browsing context.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `AddPreloadScriptParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `AddPreloadScriptResult` or a `CommandError`.
    pub async fn script_add_preload_script(
        &mut self,
        params: AddPreloadScriptParameters,
    ) -> Result<AddPreloadScriptResult, CommandError> {
        commands::script::add_preload_script(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-script-disown

    /// Disown the given handles.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `DisownParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn script_disown(
        &mut self,
        params: DisownParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::script::disown(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-script-callFunction

    /// Call a provided function with given arguments in a given realm.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `CallFunctionParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EvaluateResult` or a `CommandError`.
    pub async fn script_call_function(
        &mut self,
        params: CallFunctionParameters,
    ) -> Result<EvaluateResult, CommandError> {
        commands::script::call_function(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-script-evaluate

    /// Evaluate the given script in the given realm.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `EvaluateParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EvaluateResult` or a `CommandError`.
    pub async fn script_evaluate(
        &mut self,
        params: EvaluateParameters,
    ) -> Result<EvaluateResult, CommandError> {
        commands::script::evaluate(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-script-getRealms

    /// Return a list of all realms, optionally filtered to realms of a
    /// specific type, or to the realm associated with a navigable's active document.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `GetRealmsParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `GetRealmsResult` or a `CommandError`.
    pub async fn script_get_realms(
        &mut self,
        params: GetRealmsParameters,
    ) -> Result<GetRealmsResult, CommandError> {
        commands::script::get_realms(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-script-removePreloadScript

    /// Remove a preload script.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `RemovePreloadScriptParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn script_remove_preload_script(
        &mut self,
        params: RemovePreloadScriptParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::script::remove_preload_script(self, params).await
    }
}

// Storage commands
impl WebDriverBiDiSession {
    // https://w3c.github.io/webdriver-bidi/#command-storage-getCookies

    /// Retrieve zero or more cookies which match a set of provided parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `GetCookiesParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `GetCookiesResult` or a `CommandError`.
    pub async fn storage_get_cookies(
        &mut self,
        params: GetCookiesParameters,
    ) -> Result<GetCookiesResult, CommandError> {
        commands::storage::get_cookies(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-storage-setCookie

    /// Create a new cookie in a cookie store.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `SetCookieParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `SetCookieResult` or a `CommandError`.
    pub async fn storage_set_cookie(
        &mut self,
        params: SetCookieParameters,
    ) -> Result<SetCookieResult, CommandError> {
        commands::storage::set_cookie(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-storage-deleteCookies

    /// Remove zero or more cookies which match a set of provided parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `DeleteCookiesParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `DeleteCookiesResult` or a `CommandError`.
    pub async fn storage_delete_cookies(
        &mut self,
        params: DeleteCookiesParameters,
    ) -> Result<DeleteCookiesResult, CommandError> {
        commands::storage::delete_cookies(self, params).await
    }
}

// Input commands
impl WebDriverBiDiSession {
    // https://w3c.github.io/webdriver-bidi/#command-input-performActions

    /// Perform a specified sequence of user input actions.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `PerformActionsParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn input_perform_actions(
        &mut self,
        params: PerformActionsParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::input::perform_actions(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-input-releaseActions

    /// Reset the input state associated with the current session.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `ReleaseActionsParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn input_release_actions(
        &mut self,
        params: ReleaseActionsParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::input::release_actions(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-input-setFiles

    /// Set the files property of a given input element with type file
    /// to a set of file paths.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as a `SetFilesParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn input_set_files(
        &mut self,
        params: SetFilesParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::input::set_files(self, params).await
    }
}

// Web extension commands
impl WebDriverBiDiSession {
    // https://w3c.github.io/webdriver-bidi/#command-webExtension-install

    /// Install a web extension.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `InstallParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `InstallResult` or a `CommandError`.
    pub async fn web_extension_install(
        &mut self,
        params: InstallParameters,
    ) -> Result<InstallResult, CommandError> {
        commands::web_extension::install(self, params).await
    }

    // https://w3c.github.io/webdriver-bidi/#command-webExtension-uninstall

    /// Uninstall a web extension.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters as an `UninstallParameters` instance.
    ///
    /// # Returns
    ///
    /// A result containing the `EmptyResult` or a `CommandError`.
    pub async fn web_extension_uninstall(
        &mut self,
        params: UninstallParameters,
    ) -> Result<EmptyResult, CommandError> {
        commands::web_extension::uninstall(self, params).await
    }
}
