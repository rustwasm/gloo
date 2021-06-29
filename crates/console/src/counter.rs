//! The `console.count` and `console.countReset` functions allow you to run a counter
//! amd log it to the browser's developer tools console. You
//! call `console.count("foo")` when the counter begins, and call
//! `console.countReset("foo")` when it is to be reset.
//!
//! [See MDN for more info](https://developer.mozilla.org/en-US/docs/Web/API/Console/count).
//!
//! This API wraps both the `count` and `countReset` calls into a single type
//! named `Counter`, ensuring both are called.
//!
//! The counter is started with
//!
//! ```no_run
//! use gloo_console::Counter;
//!
//! let counter = Counter::new("foo");
//!
//! counter.count();
//! counter.count();
//! ```

use web_sys::console;

/// A console time measurement.
///
/// Dropping this will reset the counter to 0.
#[derive(Debug)]
pub struct Counter<'a> {
    label: &'a str,
}

impl<'a> Counter<'a> {
    /// Starts a console time measurement. The measurement
    /// ends when the constructed `ConsoleTimer` object is dropped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_console::Counter;
    ///
    /// let _timer = Counter::new("foo");
    /// ```
    pub fn new(label: &'a str) -> Counter<'a> {
        console::count_with_label(label);
        Counter { label }
    }

    /// Increments the counter
    pub fn count(&self) {
        console::count_with_label(self.label);
    }
}

impl<'a> Drop for Counter<'a> {
    fn drop(&mut self) {
        console::count_reset_with_label(self.label);
    }
}
