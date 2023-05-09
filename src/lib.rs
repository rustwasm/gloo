//! Gloo is a modular toolkit for building fast and reliable libraries and apps
//! with Rust and WebAssembly.

#![deny(missing_docs, missing_debug_implementations)]

// Re-exports of toolkit crates.
#[cfg(feature = "console")]
pub use gloo_console as console;
#[cfg(feature = "dialogs")]
pub use gloo_dialogs as dialogs;
#[cfg(feature = "events")]
pub use gloo_events as events;
#[cfg(feature = "file")]
pub use gloo_file as file;
#[cfg(feature = "history")]
pub use gloo_history as history;
#[cfg(feature = "net")]
pub use gloo_net as net;
#[cfg(feature = "render")]
pub use gloo_render as render;
#[cfg(feature = "storage")]
pub use gloo_storage as storage;
#[cfg(feature = "timers")]
pub use gloo_timers as timers;
#[cfg(feature = "utils")]
pub use gloo_utils as utils;
#[cfg(feature = "worker")]
pub use gloo_worker as worker;
