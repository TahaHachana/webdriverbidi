use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use futures::StreamExt;
use log::{debug, error};
use serde_json::Value;
use tokio::sync::{Mutex, oneshot};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::command_sender::notify;
use crate::events::EventType;
use crate::session::{EventHandler, MessageDirection, ObserverSlot, WebSocketReader};

const ID_FIELD: &str = "id";
const TYPE_FIELD: &str = "type";
const EVENT_TYPE_VALUE: &str = "event";
const METHOD_FIELD: &str = "method";

/// Handle incoming WebSocket messages until the connection closes or fails.
///
/// The task owns the read half of the connection and waits for each message, so
/// it uses no CPU while idle. When it stops, pending commands are dropped so
/// their callers fail immediately instead of waiting for the response timeout.
pub async fn handle_messages(
    mut stream: WebSocketReader,
    pending_commands: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    event_handlers: Arc<Mutex<HashMap<EventType, EventHandler>>>,
    observer: ObserverSlot,
) {
    while let Some(message) = stream.next().await {
        match message {
            Ok(Message::Text(text)) => match serde_json::from_str::<Value>(&text) {
                Ok(json) => {
                    notify(&observer, MessageDirection::Received, &json);
                    // debug!("Raw message JSON: {:?}", json);
                    // Command response message
                    if let Some(id) = json.get(ID_FIELD).and_then(|id| id.as_u64()) {
                        // This is a command response
                        if let Some(sender) = pending_commands.lock().await.remove(&id) {
                            // debug!("Sending JSON to receiver: {:?}", json);
                            let _ = sender.send(json);
                        }
                    // Event message
                    } else if json.get(TYPE_FIELD).and_then(|t| t.as_str())
                        == Some(EVENT_TYPE_VALUE)
                    {
                        if let Some(event_type_str) =
                            json.get(METHOD_FIELD).and_then(|method| method.as_str())
                        {
                            if let Ok(event_type) = EventType::from_str(event_type_str) {
                                let event_handlers = Arc::clone(&event_handlers);
                                let json = json.clone();
                                tokio::spawn(async move {
                                    let handlers = event_handlers.lock().await;
                                    if let Some(handler) = handlers.get(&event_type) {
                                        handler(json).await;
                                    }
                                });
                            }
                        }
                    } else {
                        error!(
                            "Received message without an 'id' field or 'type' field: {}",
                            text
                        );
                    }
                }
                Err(e) => {
                    error!("Failed to parse JSON: {:?}", e);
                }
            },
            Ok(Message::Close(frame)) => {
                debug!("WebSocket closed by the remote end: {:?}", frame);
            }
            Ok(_) => {}
            Err(e) => {
                error!("Error receiving message, closing the connection: {}", e);
                break;
            }
        }
    }
    debug!("WebSocket connection ended; failing pending commands");
    pending_commands.lock().await.clear();
}
