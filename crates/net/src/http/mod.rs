//! Wrapper around the `fetch` API.
//!
//! # Example
//!
//! ```
//! # use gloo_net::http::Request;
//! # async fn no_run() {
//! let resp = RequestWritable::get("/path")
//!     .body(None)
//!     .unwrap()
//!     .send()
//!     .await
//!     .unwrap();
//! assert_eq!(resp.status(), 200);
//! # }
//! ```

mod headers;
mod query;
mod request;
mod response;
mod method;


pub use method::Method;
pub use request::Request;
pub use response::Response;
pub use headers::Headers;
pub use query::QueryParams;
