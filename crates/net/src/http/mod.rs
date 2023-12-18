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
mod query;
mod request;
mod response;

pub use headers::Headers;
#[doc(inline)]
pub use http::Method;
pub use query::QueryParams;

pub use request::{Request, RequestBuilder};
pub use response::{IntoRawResponse, Response, ResponseBuilder};
