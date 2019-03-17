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
    /// let _timer = ConsoleTimer::new("my_label");
    /// ```
    pub fn new(label: &'a str) -> ConsoleTimer<'a> {
        console::time_with_label(label);
        ConsoleTimer { label }
    }
}

impl<'a> Drop for ConsoleTimer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.label);
    }
}

#[macro_export]
macro_rules! console_time {
    ($t:tt, $b:block) => {
        let _timer = ConsoleTimer::new($t);
        $b
    };
}
