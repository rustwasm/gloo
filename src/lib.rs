//! HTTP requests library for WASM Apps
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

mod http;
mod error;

pub use http::*;
pub use error::*;
