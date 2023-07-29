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
//! use futures::{stream, StreamExt};
//!
//! # macro_rules! console_log {
//! #    ($($expr:expr),*) => {{}};
//! # }
//! # fn no_run() {
//! let mut es = EventSource::new("http://api.example.com/ssedemo.php").unwrap();
//! let stream_1 = es.subscribe("some-event-type").unwrap();
//! let stream_2 = es.subscribe("another-event-type").unwrap();
//!
//! spawn_local(async move {
//!     let mut all_streams = stream::select(stream_1, stream_2);
//!     while let Some(Ok((event_type, msg))) = all_streams.next().await {
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
use std::fmt;
use std::fmt::Formatter;
use std::pin::Pin;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MessageEvent;

/// Wrapper around browser's EventSource API. Dropping
/// this will close the underlying event source.
#[derive(Clone)]
pub struct EventSource {
    es: web_sys::EventSource,
}

impl fmt::Debug for EventSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventSource")
            .field("url", &self.es.url())
            .field("with_credentials", &self.es.with_credentials())
            .field("ready_state", &self.state())
            .finish_non_exhaustive()
    }
}

/// Wrapper around browser's EventSource API.
#[pin_project(PinnedDrop)]
pub struct EventSourceSubscription {
    #[allow(clippy::type_complexity)]
    error_callback: Closure<dyn FnMut(web_sys::Event)>,
    es: web_sys::EventSource,
    event_type: String,
    message_callback: Closure<dyn FnMut(MessageEvent)>,
    #[pin]
    message_receiver: mpsc::UnboundedReceiver<StreamMessage>,
}

impl fmt::Debug for EventSourceSubscription {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventSourceSubscription")
            .field("event_source", &self.es)
            .field("event_type", &self.event_type)
            .finish_non_exhaustive()
    }
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

        Ok(Self { es })
    }

    /// Subscribes to listening for a specific type of event.
    ///
    /// All events for this type are streamed back given the subscription
    /// returned.
    ///
    /// The event type of "message" is a special case, as it will capture
    /// events without an event field as well as events that have the
    /// specific type `event: message`. It will not trigger on any
    /// other event type.
    pub fn subscribe(
        &mut self,
        event_type: impl Into<String>,
    ) -> Result<EventSourceSubscription, JsError> {
        let event_type = event_type.into();
        let (message_sender, message_receiver) = mpsc::unbounded();

        let message_callback: Closure<dyn FnMut(MessageEvent)> = {
            let event_type = event_type.clone();
            let sender = message_sender.clone();
            Closure::wrap(Box::new(move |e: MessageEvent| {
                let event_type = event_type.clone();
                let _ = sender.unbounded_send(StreamMessage::Message(event_type, e));
            }) as Box<dyn FnMut(MessageEvent)>)
        };

        self.es
            .add_event_listener_with_callback(
                &event_type,
                message_callback.as_ref().unchecked_ref(),
            )
            .map_err(js_to_js_error)?;

        let error_callback: Closure<dyn FnMut(web_sys::Event)> = {
            Closure::wrap(Box::new(move |e: web_sys::Event| {
                let is_connecting = e
                    .current_target()
                    .map(|target| target.unchecked_into::<web_sys::EventSource>())
                    .map(|es| es.ready_state() == web_sys::EventSource::CONNECTING)
                    .unwrap_or(false);
                if !is_connecting {
                    let _ = message_sender.unbounded_send(StreamMessage::ErrorEvent);
                };
            }) as Box<dyn FnMut(web_sys::Event)>)
        };

        self.es
            .add_event_listener_with_callback("error", error_callback.as_ref().unchecked_ref())
            .map_err(js_to_js_error)?;

        Ok(EventSourceSubscription {
            error_callback,
            es: self.es.clone(),
            event_type,
            message_callback,
            message_receiver,
        })
    }

    /// Closes the EventSource.
    ///
    /// See the [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/EventSource/close#parameters)
    /// to learn about this function
    pub fn close(mut self) {
        self.close_and_notify();
    }

    fn close_and_notify(&mut self) {
        self.es.close();
        // Fire an error event to cause all subscriber
        // streams to close down.
        if let Ok(event) = web_sys::Event::new("error") {
            let _ = self.es.dispatch_event(&event);
        }
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

impl Drop for EventSource {
    fn drop(&mut self) {
        self.close_and_notify();
    }
}

#[derive(Clone)]
enum StreamMessage {
    ErrorEvent,
    Message(String, MessageEvent),
}

impl Stream for EventSourceSubscription {
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
impl PinnedDrop for EventSourceSubscription {
    fn drop(self: Pin<&mut Self>) {
        let _ = self.es.remove_event_listener_with_callback(
            "error",
            self.error_callback.as_ref().unchecked_ref(),
        );

        let _ = self.es.remove_event_listener_with_callback(
            &self.event_type,
            self.message_callback.as_ref().unchecked_ref(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn eventsource_works() {
        let sse_echo_server_url =
            option_env!("SSE_ECHO_SERVER_URL").expect("Did you set SSE_ECHO_SERVER_URL?");

        let mut es = EventSource::new(sse_echo_server_url).unwrap();
        let mut servers = es.subscribe("server").unwrap();
        let mut requests = es.subscribe("request").unwrap();

        spawn_local(async move {
            assert_eq!(servers.next().await.unwrap().unwrap().0, "server");
            assert_eq!(requests.next().await.unwrap().unwrap().0, "request");
        });
    }

    #[wasm_bindgen_test]
    fn eventsource_connect_failure_works() {
        let mut es = EventSource::new("rubbish").unwrap();
        let mut servers = es.subscribe("server").unwrap();

        spawn_local(async move {
            // we should expect an immediate failure

            assert_eq!(
                servers.next().await,
                Some(Err(EventSourceError::ConnectionError))
            );
        })
    }
}
