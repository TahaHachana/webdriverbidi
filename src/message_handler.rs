use std::collections::HashMap;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;

use futures::stream::Stream;
use futures::task::{Context, Poll};
use log::error;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::{Mutex, oneshot};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::events::EventType;
use crate::session::EventHandler;

const ID_FIELD: &str = "id";
const TYPE_FIELD: &str = "type";
const EVENT_TYPE_VALUE: &str = "event";
const METHOD_FIELD: &str = "method";

/// Start a loop for handling incoming WebSocket messages.
pub async fn handle_messages(
    websocket_stream: Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    pending_commands: Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>,
    event_handlers: Arc<Mutex<HashMap<EventType, EventHandler>>>,
) {
    loop {
        let message = {
            // debug!("Locking the WebSocket stream mutex");
            let mut websocket_stream = websocket_stream.lock().await;

            let waker = futures::task::noop_waker();
            let mut cx = Context::from_waker(&waker);

            match Pin::new(&mut *websocket_stream).poll_next(&mut cx) {
                Poll::Ready(Some(Ok(msg))) => Some(Ok(msg)),
                Poll::Ready(Some(Err(e))) => Some(Err(e)),
                Poll::Ready(None) => None,
                Poll::Pending => None,
            }
        };

        // debug!("Received message: {:?}", message);

        match message {
            Some(Ok(Message::Text(text))) => match serde_json::from_str::<Value>(&text) {
                Ok(json) => {
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
            Some(Ok(_)) => {}
            Some(Err(e)) => {
                error!("Error receiving message: {}", e);
            }
            None => {}
        }
    }
}
