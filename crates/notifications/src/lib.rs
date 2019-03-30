//! Displaying notifications on the web.
//!
//! This API comes in two flavors: A callback style and `Future`s API.
//!
//! Before a notification can be displayed, the user of the browser has to give his permission.
//!
//! ## 1. Callback style
//!
//! ```rust
//! use gloo_notifications::Notification;
//!
//! Notification::request_permission_map(|mut builder| {
//!     builder.title("Notification title").show();
//! });
//!
//! Notification::request_permission_map_or(|mut builder| {
//!     builder.title("Notification title")
//!         .body("Notification body")
//!         .show();
//! }, |_| {
//!     // in case the permission is denied
//! });
//!
//! // short form, if you only need a title
//! Notification::request_permission_with_title("Notification title");
//! ```
//!
//! ## 2. `Future` API:
//!
//! ```rust
//! use gloo_notifications::Notification;
//!
//! Notification::request_permission()
//!     .map(|mut builder| {
//!         builder.title("Notification title").show();
//!     })
//!     .map_err(|_| {
//!         // in case the permission is denied
//!     })
//! ```
//!
//! ## Adding event listeners
//!
//! This part of the API is **unstable**!
//!
//! ```rust
//! use gloo_notifications::Notification;
//!
//! Notification::request_permission_map(|mut builder| {
//!     let notification = builder
//!         .title("Notification title")
//!         .show();
//!
//!     notification.onclick(|_| { ... });
//! })
//! ```
//!
//! ## Macro
//!
//! ```rust
//! use gloo_notifications::{Notification, notification};
//!
//! // requests permission, then displays the notification
//! // and adds a "click" event listener
//! notification! {
//!     title: "Hello World",
//!     body: "Foo",
//!     icon: "/assets/notification.png";
//!     onclick: |_| {}
//! }
//! ```

#![cfg_attr(feature = "futures", doc = "```no_run")]
#![cfg_attr(not(feature = "futures"), doc = "```ignore")]
#![deny(missing_docs, missing_debug_implementations)]

#[cfg(feature = "futures")]
extern crate futures_rs as futures;

use wasm_bindgen::{closure::Closure, JsCast, JsValue, UnwrapThrowExt};
pub use web_sys::{NotificationDirection, NotificationOptions, NotificationPermission};

#[cfg(feature = "futures")]
pub mod future;

mod builder;
pub use builder::NotificationBuilder;

/// A notification.
///
/// This struct can not be created directly, you have to request permission first.
#[repr(transparent)]
#[derive(Debug)]
pub struct Notification {
    sys_notification: web_sys::Notification,
}

/// Requests permission, then displays the notification if the permission was granted.
///
/// ### Example
///
/// ```rust
/// notification! {
///     title: "Hello World",
///     body: "Foo",
/// }
/// ```
///
/// The identifiers are the same as the setter methods of `NotificationBuilder`.
///
/// The macro can also add a "click" event listener. Simply add a semicolon `;` after the
/// properties and add a `onclick` property:
///
///
/// ```rust
/// notification! {
///     title: "Hello World",
///     body: "Foo" ;
///     onclick: |_| { ... }
/// }
/// ```
#[macro_export]
macro_rules! notification {
    ( $($k:ident : $v:expr),* $(,)? ) => (
        Notification::request_permission_map(|mut builder| {
            builder
            $( .$k($v) )*
            .show();
        });
    );
    ( $($k:ident : $v:expr),* ; onclick: $e:expr $(,)? ) => (
        Notification::request_permission_map(|mut builder| {
            let o = builder
            $( .$k($v) )*
            .show();
            o.onclick(Some($e));
        });
    );
}

impl Notification {
    fn new<'a>(builder: &'a NotificationBuilder) -> Notification {
        let (title, sys_builder) = builder.get_inner();
        let sys_notification =
            web_sys::Notification::new_with_options(title, sys_builder).unwrap_throw();
        Notification { sys_notification }
    }

    fn with_title<'a>(title: &'a str) -> Notification {
        let sys_builder = &NotificationOptions::new();
        let sys_notification =
            web_sys::Notification::new_with_options(title, sys_builder).unwrap_throw();
        Notification { sys_notification }
    }

    /// This returns the permission to display notifications, which is one of the following values:
    ///
    /// - `default`: The user has neither granted, nor denied his permission.
    ///     Calling `Notification::request_permission()` displays a dialog window.
    /// - `granted`: You are allowed to display notifications.
    ///     Calling `Notification::request_permission()` succeeds immediately.
    /// - `denied`: You are forbidden to display notifications.
    ///     Calling `Notification::request_permission()` fails immediately.
    #[inline]
    pub fn permission() -> NotificationPermission {
        web_sys::Notification::permission()
    }

    /// Requests permission to display notifications, and asynchronously calls `f`
    /// with a new `NotificationBuilder`, when the permission is granted.
    ///
    /// If the permission is denied, nothing happens.
    pub fn request_permission_map<F>(mut f: F)
    where
        F: FnMut(NotificationBuilder) + 'static,
    {
        let resolve = Closure::once(move |_| f(NotificationBuilder::new()));

        web_sys::Notification::request_permission()
            .unwrap_throw()
            .then(&resolve);
    }

    /// Requests permission to display notifications, and asynchronously calls
    ///
    /// - `f(NotificationBuilder)`, if the permission is granted
    /// - `g()`, if the permission is denied
    pub fn request_permission_map_or<Ok, Err>(mut f: Ok, g: Err)
    where
        Ok: FnMut(NotificationBuilder) + 'static,
        Err: FnMut(JsValue) + 'static,
    {
        let resolve = Closure::once(move |_| f(NotificationBuilder::new()));
        let reject = Closure::once(g);

        web_sys::Notification::request_permission()
            .unwrap_throw()
            .then2(&resolve, &reject);
    }


    /// Requests permission to display notifications,
    /// and displays a notification with a title, if the permission is granted.
    ///
    /// If the permission is denied, nothing happens.
    pub fn request_permission_with_title(title: &'static str) {
        let resolve = Closure::once(move |_| {
            Notification::with_title(title);
        });

        web_sys::Notification::request_permission()
            .unwrap_throw()
            .then(&resolve);
    }


    /// Sets the "click" event listener
    pub fn onclick<F>(&self, listener: Option<F>) -> &Self
    where
        F: Fn(JsValue) + 'static,
    {
        match listener {
            Some(f) => {
                let boxed: Box<dyn Fn(JsValue)> = Box::new(f);
                self.sys_notification
                    .set_onclick(Some(Closure::wrap(boxed).as_ref().unchecked_ref()));
            }
            None => self.sys_notification.set_onclick(None),
        }
        self
    }

    /// Closes the notification
    #[inline]
    pub fn close(&self) {
        self.sys_notification.close()
    }
}
