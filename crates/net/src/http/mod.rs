//! Wrapper around the `fetch` API.
//!
//! # Example
//!
//! ```
//! # use gloo_net::http::Request;
//! # async fn no_run() {
//! let resp = Request::get("/path")
//!     .send()
//!     .await
//!     .unwrap();
//! assert_eq!(resp.status(), 200);
//! # }
//! ```

mod headers;
mod method;
mod query;
mod request;
mod response;

pub use headers::Headers;
pub use method::Method;
pub use query::QueryParams;
pub use request::Request;
pub use response::Response;
