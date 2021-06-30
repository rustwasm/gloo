//! The `console.time` and `console.timeEnd` functions allow you to log the
//! timing of named operations to the browser's developer tools console. You
//! call `console.time("foo")` when the operation begins, and call
//! `console.timeEnd("foo")` when it finishes.
//!
//! Additionally, these measurements will show up in your browser's profiler's
//! "timeline" or "waterfall" view.
//!
//! [See MDN for more info](https://developer.mozilla.org/en-US/docs/Web/API/console#Timers).
//!
//! This API wraps both the `time` and `timeEnd` calls into a single type
//! named `ConsoleTimer`, ensuring both are called.
//!
//! ## Scoped Measurement
//!
//! Wrap code to be measured in a closure with [`Timer::scope`].
//!
//! ```no_run
//! use gloo_console::Timer;
//!
//! let value = Timer::scope("foo", || {
//!     // Place code to be measured here
//!     // Optionally return a value.
//! });
//! ```
//!
//! ## RAII-Style Measurement
//!
//! For scenarios where [`Timer::scope`] can't be used, like with
//! asynchronous operations, you can use `ConsoleTimer::new` to create a timer.
//! The measurement ends when the timer object goes out of scope / is dropped.
//!
//! ```no_run
//! use gloo_console::Timer;
//! use gloo_timers::callback::Timeout;
//!
//! // Start timing a new operation.
//! let timer = Timer::new("foo");
//!
//! // And then asynchronously finish timing.
//! let timeout = Timeout::new(1_000, move || {
//!     drop(timer);
//! });
//! ```

use web_sys::console;

/// A console time measurement.
///
/// See [`Timer::scope`] for starting a labeled time measurement
/// of code wrapped in a closure.
#[derive(Debug)]
pub struct Timer<'a> {
    label: &'a str,
}

impl<'a> Timer<'a> {
    /// Starts a console time measurement. The measurement
    /// ends when the constructed `ConsoleTimer` object is dropped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_console::Timer;
    ///
    /// let _timer = Timer::new("foo");
    /// ```
    pub fn new(label: &'a str) -> Timer<'a> {
        console::time_with_label(label);
        Timer { label }
    }

    /// Starts a scoped console time measurement
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_console::Timer;
    ///
    /// let value = Timer::scope("foo", || {
    ///     // Code to measure here
    /// });
    /// ```
    pub fn scope<F, T>(label: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _timer = Timer::new(label);
        f()
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.label);
    }
}
