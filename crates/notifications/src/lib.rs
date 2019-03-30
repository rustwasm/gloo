//! Displaying notifications on the web.
//!
//! This API comes in two flavors:
//!
//! 1. a callback style (that more directly mimics the JavaScript APIs), and
//! 2. a `Future`s API.
//!
//! Before a notification can be displayed, the user of the browser has to give his permission.
//!
//! Because the permission can also be withdrawn, permission *must* be checked
//! every time a notification is displayed.

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

/// A notification. This struct can not be created directly,
/// because you might not have permission to show a notification.
///
/// ## 1. Callback API
///
/// ```rust
/// use gloo_notifications::Notification;
///
/// Notification::request_permission_map(|mut builder| {
///     let _notification = builder
///         .title("Notification title")
///         .show();
/// });
///
/// // or
///
/// Notification::request_permission_map_or(|mut builder| {
///     let _notification = builder
///         .title("Notification title")
///         .show();
/// }, |_| {
///     // in case the permission is denied
/// });
/// ```
///
/// ## 2. `Future` API:
///
/// ```rust
/// use gloo_notifications::Notification;
///
/// Notification::request_permission()
///     .map(|mut builder| {
///         let _notification = builder
///             .title("Notification title")
///             .show();
///     })
///     .map_err(|_| {
///         // in case the permission is denied
///     })
/// ```
///
/// ## Adding event listeners
///
/// ```rust
/// use gloo_notifications::Notification;
///
/// Notification::request_permission()
///     .map(|mut builder| {
///         let notification = builder
///             .title("Notification title")
///             .show();
///
///         on(&notification, |e: ClickEvent| {});
///         on(&notification, |e: ShowEvent | {});
///         on(&notification, |e: ErrorEvent| {});
///         on(&notification, |e: CloseEvent| {});
///     })
/// ```
#[repr(transparent)]
#[derive(Debug)]
pub struct Notification {
    sys_notification: web_sys::Notification,
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

    /// Requests permission to display notifications,
    /// and displays a notification with a title, if the permission is granted.
    ///
    /// If the permission is denied, `f()` is called.
    pub fn request_permission_with_title_or<Err>(title: &'static str, f: Err)
    where
        Err: FnMut(JsValue) + 'static
    {
        let resolve = Closure::once(move |_| {
            Notification::with_title(title);
        });
        let reject = Closure::once(f);

        web_sys::Notification::request_permission()
            .unwrap_throw()
            .then2(&resolve, &reject);
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
