/*!

TODO

Add examples and docs

 */

#![deny(missing_docs, missing_debug_implementations)]

use web_sys::console;

/// A console time measurement
///
/// See `ConsoleTimer::new` for starting a labeled time measurement.
///
/// Measurement ends when the constructed `ConsoleTimer` object is dropped.
#[derive(Debug)]
pub struct ConsoleTimer<'a> {
    label: &'a str,
}

impl<'a> ConsoleTimer<'a> {
    /// Starts a console time measruement.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_console::ConsoleTimer;
    ///
    /// let _timer = ConsoleTimer::new("foo");
    /// ```
    pub fn new(label: &'a str) -> ConsoleTimer<'a> {
        console::time_with_label(label);
        ConsoleTimer { label }
    }

    /// Starts a scoped console time measurement
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_console::ConsoleTimer;
    ///
    /// let value = ConsoleTimer::scope("foo", || {
    ///     // ...
    /// });
    /// ```
    pub fn scope<F, T>(label: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _timer = ConsoleTimer::new(label);
        f()
    }
}

impl<'a> Drop for ConsoleTimer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.label);
    }
}
