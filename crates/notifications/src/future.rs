//! This module provides the `Notification::request_permission()` function,
//! which returns a `futures_rs::Future`.

extern crate futures_rs as futures;

use futures::Future;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;

use crate::{Notification, NotificationBuilder};

impl Notification {
    /// ```rust
    /// use gloo_notifications::Notification;
    ///
    /// Notification::request_permission()
    ///     .map(|mut builder| {
    ///         let _notification = builder
    ///             .title("Hello World")
    ///             .show();
    ///     })
    ///     .map_err(|_| {
    ///         // in case the permission is denied
    ///     })
    /// ```
    ///
    #[must_use = "futures do nothing unless polled"]
    pub fn request_permission<'a>() -> impl Future<Item = NotificationBuilder<'a>, Error = JsValue>
    {
        let promise = web_sys::Notification::request_permission().unwrap_throw();

        JsFuture::from(promise).map(|_| NotificationBuilder::new())
    }
}
