use crate::{js_to_error, JsError};
use futures::channel::mpsc;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures::StreamExt;
pub use gloo::file::Blob;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent};

/// Wrapper around browser's WebSocket API.
#[allow(missing_debug_implementations)]
pub struct WebSocket {
    /// Raw websocket instance
    pub websocket: web_sys::WebSocket,
    /// Channel's receiver component used to receive messages from the WebSocket
    pub receiver: UnboundedReceiver<Result<Message, WebSocketError>>,
    /// Channel's sender component used to send messages over the WebSocket
    pub sender: UnboundedSender<Message>,
}

/// Message received from WebSocket.
#[derive(Debug)]
pub enum Message {
    /// String message
    Text(String),
    /// ArrayBuffer parsed into bytes
    Bytes(Vec<u8>),
}

impl WebSocket {
    /// Establish a WebSocket connection.
    pub fn open(url: &str) -> Result<Self, crate::error::Error> {
        let ws = web_sys::WebSocket::new(url).map_err(js_to_error)?;

        let (internal_sender, receiver) = mpsc::unbounded();
        let (sender, mut internal_receiver) = mpsc::unbounded();

        let (notify_sender, mut notify_receiver) = mpsc::unbounded();

        let open_callback: Closure<dyn FnMut()> = {
            Closure::wrap(Box::new(move || {
                notify_sender.unbounded_send(()).unwrap();
            }) as Box<dyn FnMut()>)
        };

        ws.set_onopen(Some(open_callback.as_ref().unchecked_ref()));
        open_callback.forget();

        let message_callback: Closure<dyn FnMut(MessageEvent)> = {
            let sender = internal_sender.clone();
            Closure::wrap(Box::new(move |e: MessageEvent| {
                sender
                    .unbounded_send(Ok(parse_message(e)))
                    .expect("message send")
            }) as Box<dyn FnMut(MessageEvent)>)
        };

        ws.set_onmessage(Some(message_callback.as_ref().unchecked_ref()));
        message_callback.forget();

        let error_callback: Closure<dyn FnMut(ErrorEvent)> = {
            let sender = internal_sender.clone();
            Closure::wrap(Box::new(move |e: ErrorEvent| {
                sender.unbounded_send(parse_error(e)).expect("message send")
            }) as Box<dyn FnMut(ErrorEvent)>)
        };

        ws.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        error_callback.forget();

        let fut = {
            let ws = ws.clone();
            let sender = internal_sender;
            async move {
                notify_receiver.next().await;
                while let Some(message) = internal_receiver.next().await {
                    let result = match message {
                        Message::Bytes(bytes) => ws.send_with_u8_array(&bytes),
                        Message::Text(message) => ws.send_with_str(&message),
                    };

                    if let Err(e) = result {
                        match js_to_error(e) {
                            crate::Error::JsError(error) => sender
                                .unbounded_send(Err(WebSocketError::JsError(error)))
                                .unwrap(),
                            _ => unreachable!(),
                        }
                    }
                }
            }
        };
        wasm_bindgen_futures::spawn_local(fut);

        Ok(Self {
            websocket: ws,
            receiver,
            sender,
        })
    }
}

fn parse_error(event: ErrorEvent) -> Result<Message, WebSocketError> {
    Err(WebSocketError::ConnectionError {
        message: event.message(),
    })
}

/// Error from a WebSocket
#[derive(Debug, thiserror::Error)]
pub enum WebSocketError {
    /// This is created from [`ErrorEvent`] received from `onerror` listener of the WebSocket.
    #[error("{message}")]
    ConnectionError {
        /// The error message.
        message: String,
    },
    /// Error from JavaScript
    #[error("{0}")]
    JsError(JsError),
}

fn parse_message(event: MessageEvent) -> Message {
    if let Ok(array_buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
        let array = js_sys::Uint8Array::new(&array_buffer);
        Message::Bytes(array.to_vec())
    } else if let Ok(txt) = event.data().dyn_into::<js_sys::JsString>() {
        Message::Text(String::from(&txt))
    } else {
        unreachable!("message event, received Unknown: {:?}", event.data());
    }
}
