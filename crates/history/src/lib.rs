//! A module that provides universal session history and location information.

// use std::borrow::Cow;
// use std::cell::RefCell;
// use std::rc::Rc;

// use gloo_events::EventListener;
// use gloo_utils::window;
// use serde::de::DeserializeOwned;
// use serde::Serialize;
// use wasm_bindgen::{JsValue, UnwrapThrowExt};

mod error;
mod history;
mod listener;
mod location;

pub use error::{HistoryError, HistoryResult};
pub use history::History;
pub use listener::HistoryListener;
pub use location::Location;
