//! HTTP requests library for WASM apps. It provides idiomatic Rust bindings for the `web_sys` `fetch` API.
//!
//! # Example
//!
//! ```no_run
//! # use reqwasm::Request;
//! let resp = Request::get("/path")
//!     .send()
//!     .await
//!     .unwrap();
//! assert_eq!(resp.status(), 200);
//! ```

#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations)]

mod error;
mod http;

pub use error::*;
pub use http::*;
