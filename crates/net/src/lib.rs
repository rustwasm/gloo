//! HTTP requests library for WASM apps. It provides idiomatic Rust bindings for the `web_sys`
//! `fetch` and `WebSocket` API.
//!
//! See module level documentation for [`http`] and [`websocket`] to learn more.

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
#[cfg(feature = "eventsource")]
#[cfg_attr(docsrs, doc(cfg(feature = "eventsource")))]
pub mod eventsource;
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub mod http;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub mod websocket;

pub use error::*;

#[cfg(test)]
/// Checks if a slice is strictly sorted.
///
/// Strictly sorted means that each element is _less_ than the next.
/// 
/// TODO: Use `is_sorted` when it becomes stable.
fn is_strictly_sorted<T: Ord>(slice: &[T]) -> bool {
    slice.iter().zip(slice.iter().skip(1)).all(|(a, b)| a < b)
}
