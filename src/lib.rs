//! Gloo is a modular toolkit for building fast and reliable libraries and apps
//! with Rust and WebAssembly.

#![deny(missing_docs, missing_debug_implementations)]

// Re-exports of toolkit crates.
pub use gloo_console as console;
pub use gloo_dialogs as dialogs;
pub use gloo_events as events;
pub use gloo_file as file;
pub use gloo_history as history;
pub use gloo_net as net;
pub use gloo_render as render;
pub use gloo_storage as storage;
pub use gloo_timers as timers;
pub use gloo_utils as utils;
pub use gloo_worker as worker;
