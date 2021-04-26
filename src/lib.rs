//! HTTP requests library for WASM apps. It provides idiomatic Rust bindings for the `web_sys`
//! `fetch` and `WebSocket` API.
//!
//! See module level documentation for [`http`] and [`websocket`] to learn more.

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations
)]

mod error;
pub mod http;
pub mod websocket;

pub use error::*;
