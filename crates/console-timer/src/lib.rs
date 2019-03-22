/*!

The `console.time` and `console.timeEnd` functions allow you to log the
timing of named operations to the browser's developer tools console. You
call `console.time("foo")` when the operation begins, and call
`console.timeEnd("foo")` when it finishes.

Additionally, these measurements will show up in your browser's profiler's
"timeline" or "waterfall" view.

[See MDN for more info](https://developer.mozilla.org/en-US/docs/Web/API/console#Timers).

This API wraps both the `time` and `timeEnd` calls into a single type
named `ConsoleTimer`, ensuring both are called.

## Scoped measurement

Wrap code to be measured in a closure with `ConsoleTimer::scope`.

```no_run
use gloo_console_timer::ConsoleTimer;

let value = ConsoleTimer::scope("foo", || {
    // Place code to be measured here
    // Optionally return a value.
});
```

## RAII-style measurement

For scenarios where `ConsoleTimer::scope` can't be used, like with
asynchronous operations, you can use `ConsoleTimer::new` to create a timer.
The measurement ends when the timer object goes out of scope / is dropped.

```no_run
use gloo_console_timer::ConsoleTimer;
use gloo_timers::Timeout;

let timeout = Timeout::new(1_000, move || {
    let _timer = ConsoleTimer::new("foo");
    // Do some work which will be measured
});
```

 */

#![deny(missing_docs, missing_debug_implementations)]

use web_sys::console;

/// A console time measurement.
///
/// See `ConsoleTimer::scope` for starting a labeled time measurement
/// of code wrapped in a closure.
#[derive(Debug)]
pub struct ConsoleTimer<'a> {
    label: &'a str,
}

impl<'a> ConsoleTimer<'a> {
    /// Starts a console time measurement. The measurement
    /// ends when the constructed `ConsoleTimer` object is dropped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_console_timer::ConsoleTimer;
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
    /// use gloo_console_timer::ConsoleTimer;
    ///
    /// let value = ConsoleTimer::scope("foo", || {
    ///     // Code to measure here
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
