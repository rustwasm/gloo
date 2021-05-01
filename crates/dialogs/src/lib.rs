//! This crate provides wrapper for `alert`, `prompt` and `confirm` functions.
//! `web-sys` provides a raw API which is hard to use. This crate provides an easy-to-use,
//! idiomatic Rust API for these functions.
//!
//! See the documentation for [`alert`], [`prompt`] and [`confirm`] for more information.

use wasm_bindgen::prelude::*;

/// Calls the alert function.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Window/alert)
pub fn alert(message: &str) {
    window().alert_with_message(message).unwrap_throw()
}

/// Calls the confirm function.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Window/confirm)
pub fn confirm(message: &str) -> bool {
    window().confirm_with_message(message).unwrap_throw()
}

/// Calls the `prompt` function.
///
/// A default value can be supplied which will be returned if the user doesn't input anything.
/// This function will return `None` if the value of `default` is `None` and the user cancels
/// the operation.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Window/prompt)
pub fn prompt(message: &str, default: Option<&str>) -> Option<String> {
    match default {
        Some(default) => window()
            .prompt_with_message_and_default(message, default)
            .expect_throw("can't read input"),
        None => window()
            .prompt_with_message(message)
            .expect_throw("can't read input"),
    }
}

#[inline]
fn window() -> web_sys::Window {
    web_sys::window().expect_throw("can't access window")
}
