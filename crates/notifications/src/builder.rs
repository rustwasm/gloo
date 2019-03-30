use crate::Notification;
use wasm_bindgen::JsValue;
use web_sys::{NotificationDirection, NotificationOptions};

/// A builder for a `Notification`.
///
/// The builder is turned into a `Notification` by calling `.show()`,
/// which displays the notifcation on the screen.
///
/// Example:
///
/// ```rust
/// use gloo_notifications::Notification;
///
/// Notification::request_permission()
///     .map(|mut builder| {
///         let _notification = builder
///             .title("Notification title")
///             .body("Notification body")
///             .show();
///     })
/// ```
#[derive(Debug)]
pub struct NotificationBuilder<'a> {
    title: &'a str,
    sys_builder: NotificationOptions,
}

impl<'a> NotificationBuilder<'a> {
    #[inline]
    pub(crate) fn new() -> Self {
        NotificationBuilder {
            title: "",
            sys_builder: NotificationOptions::new(),
        }
    }

    #[inline]
    pub(crate) fn get_inner(&self) -> (&str, &NotificationOptions) {
        (self.title, &self.sys_builder)
    }

    /// Sets the title of the notification
    #[inline]
    #[must_use = "You have to call .show() to display the notification"]
    pub fn title(&mut self, title: &'a str) -> &mut Self {
        self.title = title;
        self
    }

    /// Sets the body text of the notification
    #[inline]
    #[must_use = "You have to call .show() to display the notification"]
    pub fn body(&mut self, body: &str) -> &mut Self {
        self.sys_builder.body(body);
        self
    }

    /// Sets the data of the notification
    #[inline]
    #[must_use = "You have to call .show() to display the notification"]
    pub fn data(&mut self, val: &JsValue) -> &mut Self {
        self.sys_builder.data(val);
        self
    }

    /// Sets the direction of the notification, which is either Auto, Ltr or Rtl.
    #[inline]
    #[must_use = "You have to call .show() to display the notification"]
    pub fn dir(&mut self, dir: NotificationDirection) -> &mut Self {
        self.sys_builder.dir(dir);
        self
    }

    /// Sets the icon of the notification
    #[inline]
    #[must_use = "You have to call .show() to display the notification"]
    pub fn icon(&mut self, val: &str) -> &mut Self {
        self.sys_builder.icon(val);
        self
    }

    /// Sets the language of the notification
    #[inline]
    #[must_use = "You have to call .show() to display the notification"]
    pub fn lang(&mut self, val: &str) -> &mut Self {
        self.sys_builder.lang(val);
        self
    }

    /// Sets the requireInteraction property.
    ///
    /// If set to `true`, the notification stays visible until the user activates or closes it.
    #[inline]
    #[must_use = "You have to call .show() to display the notification"]
    pub fn require_interaction(&mut self, val: bool) -> &mut Self {
        self.sys_builder.require_interaction(val);
        self
    }

    /// Sets the tag of the notification
    #[inline]
    #[must_use = "You have to call .show() to display the notification"]
    pub fn tag(&mut self, val: &str) -> &mut Self {
        self.sys_builder.tag(val);
        self
    }

    /// Returns a new Notification from this builder
    /// and displays it in the browser, if the permission is granted
    #[inline]
    pub fn show(&self) -> Notification {
        Notification::new(self)
    }
}
