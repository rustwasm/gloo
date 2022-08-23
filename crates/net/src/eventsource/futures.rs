//! A wrapper around the `EventSource` API using the Futures API to be used with async rust.
//!
//! EventSource is similar to WebSocket with the major differences being:
//!
//! * they are a one-way stream of server generated events
//! * their connection is managed entirely by the browser
//! * their data is slightly more structured including an id, type and data
//!
//! EventSource is therefore suitable for simpler scenarios than WebSocket.
//!
//! See the [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events) to learn more.
//!
//! # Example
//!
//! ```rust
//! use gloo_net::eventsource::futures::EventSource;
//! use wasm_bindgen_futures::spawn_local;
//! use futures::StreamExt;
//!
//! # macro_rules! console_log {
//! #    ($($expr:expr),*) => {{}};
//! # }
//! # fn no_run() {
//! let mut es = EventSource::new("http://api.example.com/ssedemo.php").unwrap();
//! es.subscribe_event("some-event-type").unwrap();
//! es.subscribe_event("another-event-type").unwrap();
//!
//! spawn_local(async move {
//!     while let Some(Ok((event_type, msg))) = es.next().await {
//!         console_log!(format!("1. {}: {:?}", event_type, msg))
//!     }
//!     console_log!("EventSource Closed");
//! })
//! # }
//! ```
use crate::eventsource::{EventSourceError, State};
use crate::js_to_js_error;
use futures_channel::mpsc;
use futures_core::{ready, Stream};
use gloo_utils::errors::JsError;
use pin_project::{pin_project, pinned_drop};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MessageEvent;

/// Wrapper around browser's EventSource API.
#[allow(missing_debug_implementations)]
#[pin_project(PinnedDrop)]
pub struct EventSource {
    es: web_sys::EventSource,
    message_sender: mpsc::UnboundedSender<StreamMessage>,
    #[pin]
    message_receiver: mpsc::UnboundedReceiver<StreamMessage>,
    #[allow(clippy::type_complexity)]
    closures: Arc<
        Mutex<(
            HashMap<String, Closure<dyn FnMut(MessageEvent)>>,
            Closure<dyn FnMut(web_sys::Event)>,
        )>,
    >,
}

impl EventSource {
    /// Establish an EventSource.
    ///
    /// This function may error in the following cases:
    /// - The connection url is invalid
    ///
    /// The error returned is [`JsError`]. See the
    /// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/EventSource/EventSource#exceptions_thrown)
    /// to learn more.
    pub fn new(url: &str) -> Result<Self, JsError> {
        let es = web_sys::EventSource::new(url).map_err(js_to_js_error)?;

        let (message_sender, message_receiver) = mpsc::unbounded();

        let error_callback: Closure<dyn FnMut(web_sys::Event)> = {
            let sender = message_sender.clone();
            Closure::wrap(Box::new(move |e: web_sys::Event| {
                let sender = sender.clone();
                let is_connecting = e
                    .current_target()
                    .and_then(|target| target.dyn_into::<web_sys::EventSource>().ok())
                    .map(|es| es.ready_state() == web_sys::EventSource::CONNECTING)
                    .unwrap_or(false);
                if !is_connecting {
                    let _ = sender.unbounded_send(StreamMessage::ErrorEvent);
                };
            }) as Box<dyn FnMut(web_sys::Event)>)
        };

        es.set_onerror(Some(error_callback.as_ref().unchecked_ref()));

        Ok(Self {
            es,
            message_sender,
            message_receiver,
            closures: Arc::new(Mutex::new((HashMap::new(), error_callback))),
        })
    }

    /// Subscribes to listening for a specific type of event. Can be
    /// called multiple times. Subscribing again with an event type
    /// that has already been subscribed is benign.
    ///
    /// All event types are streamed back with the element of the stream
    /// being a tuple of event type and message event.
    ///
    /// The event type of "message" is a special case, as it will capture
    /// events without an event field as well as events that have the
    /// specific type `event: message`. It will not trigger on any
    /// other event type.
    pub fn subscribe_event(&mut self, event_type: &str) -> Result<(), JsError> {
        let event_type = event_type.to_string();
        match self.closures.lock() {
            Ok(mut closures) => {
                let (message_callbacks, _) = closures.deref_mut();

                if let Entry::Vacant(entry) = message_callbacks.entry(event_type.clone()) {
                    let message_callback: Closure<dyn FnMut(MessageEvent)> = {
                        let sender = self.message_sender.clone();
                        Closure::wrap(Box::new(move |e: MessageEvent| {
                            let sender = sender.clone();
                            let event_type = event_type.clone();
                            let _ = sender.unbounded_send(StreamMessage::Message(event_type, e));
                        }) as Box<dyn FnMut(MessageEvent)>)
                    };

                    self.es
                        .add_event_listener_with_callback(
                            entry.key(),
                            message_callback.as_ref().unchecked_ref(),
                        )
                        .map_err(js_to_js_error)?;

                    entry.insert(message_callback);
                }
                Ok(())
            }
            Err(e) => Err(js_sys::Error::new(&format!("Failed to subscribe: {}", e)).into()),
        }
    }

    /// Unsubscribes from listening for a specific type of event. Unsubscribing
    /// multiple times is benign.
    pub fn unsubscribe_event(&mut self, event_type: &str) -> Result<(), JsError> {
        match self.closures.lock() {
            Ok(mut closures) => {
                let (message_callbacks, _) = closures.deref_mut();

                if let Some(message_callback) = message_callbacks.remove(event_type) {
                    self.es
                        .remove_event_listener_with_callback(
                            event_type,
                            message_callback.as_ref().unchecked_ref(),
                        )
                        .map_err(js_to_js_error)?;
                }
                Ok(())
            }
            Err(e) => Err(js_sys::Error::new(&format!("Failed to subscribe: {}", e)).into()),
        }
    }

    /// Closes the EventSource.
    ///
    /// See the [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/EventSource/close#parameters)
    /// to learn about this function
    pub fn close(self) {
        self.es.close();
    }

    /// The current state of the EventSource.
    pub fn state(&self) -> State {
        let ready_state = self.es.ready_state();
        match ready_state {
            0 => State::Connecting,
            1 => State::Open,
            2 => State::Closed,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
enum StreamMessage {
    ErrorEvent,
    Message(String, MessageEvent),
}

impl Stream for EventSource {
    type Item = Result<(String, MessageEvent), EventSourceError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let msg = ready!(self.project().message_receiver.poll_next(cx));
        match msg {
            Some(StreamMessage::Message(event_type, msg)) => {
                Poll::Ready(Some(Ok((event_type, msg))))
            }
            Some(StreamMessage::ErrorEvent) => {
                Poll::Ready(Some(Err(EventSourceError::ConnectionError)))
            }
            None => Poll::Ready(None),
        }
    }
}

#[pinned_drop]
impl PinnedDrop for EventSource {
    fn drop(self: Pin<&mut Self>) {
        self.es.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    const SSE_ECHO_SERVER_URL: &str = env!("SSE_ECHO_SERVER_URL");

    #[wasm_bindgen_test]
    fn eventsource_works() {
        let mut es = EventSource::new(SSE_ECHO_SERVER_URL).unwrap();
        es.subscribe_event("server").unwrap();
        es.subscribe_event("request").unwrap();

        spawn_local(async move {
            assert_eq!(es.next().await.unwrap().unwrap().0, "server");
            assert_eq!(es.next().await.unwrap().unwrap().0, "request");
            es.unsubscribe_event("request").unwrap();
        });
    }

    #[wasm_bindgen_test]
    fn eventsource_close_works() {
        let mut es = EventSource::new("rubbish").unwrap();

        spawn_local(async move {
            // we should expect an immediate failure

            assert_eq!(
                es.next().await,
                Some(Err(EventSourceError::ConnectionError))
            );
        })
    }
}
