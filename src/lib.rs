//! Gloo is a modular toolkit for building fast and reliable libraries and apps
//! with Rust and WebAssembly.

#![deny(missing_docs, missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

// Re-exports of toolkit crates.
#[cfg(feature = "console")]
#[cfg_attr(docsrs, doc(cfg(feature = "console")))]
#[doc(inline)]
pub use gloo_console as console;
#[cfg(feature = "dialogs")]
#[cfg_attr(docsrs, doc(cfg(feature = "dialogs")))]
#[doc(inline)]
pub use gloo_dialogs as dialogs;
#[cfg(feature = "events")]
#[cfg_attr(docsrs, doc(cfg(feature = "events")))]
#[doc(inline)]
pub use gloo_events as events;
#[cfg(feature = "file")]
#[cfg_attr(docsrs, doc(cfg(feature = "file")))]
#[doc(inline)]
pub use gloo_file as file;
#[cfg(feature = "history")]
#[cfg_attr(docsrs, doc(cfg(feature = "history")))]
#[doc(inline)]
pub use gloo_history as history;
#[cfg(feature = "net")]
#[cfg_attr(docsrs, doc(cfg(feature = "net")))]
#[doc(inline)]
pub use gloo_net as net;
#[cfg(feature = "render")]
#[cfg_attr(docsrs, doc(cfg(feature = "render")))]
#[doc(inline)]
pub use gloo_render as render;
#[cfg(feature = "storage")]
#[cfg_attr(docsrs, doc(cfg(feature = "storage")))]
#[doc(inline)]
pub use gloo_storage as storage;
#[cfg(feature = "timers")]
#[cfg_attr(docsrs, doc(cfg(feature = "timers")))]
#[doc(inline)]
pub use gloo_timers as timers;
#[cfg(feature = "utils")]
#[cfg_attr(docsrs, doc(cfg(feature = "utils")))]
#[doc(inline)]
pub use gloo_utils as utils;
#[cfg(feature = "worker")]
#[cfg_attr(docsrs, doc(cfg(feature = "worker")))]
#[doc(inline)]
pub use gloo_worker as worker;
