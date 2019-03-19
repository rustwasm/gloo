/*!

Working with timers on the Web: `setTimeout` and `setInterval`.

These APIs come in two flavors:

1. a callback style (that more directly mimics the JavaScript APIs), and
2. a `Future`s and `Stream`s API.

## Timeouts

Timeouts fire once after a period of time (measured in milliseconds).

### Timeouts with a Callback Function

```no_run
use gloo_timers::Timeout;

let timeout = Timeout::new(1_000, move || {
    // Do something after the one second timeout is up!
});

// Since we don't plan on cancelling the timeout, call `forget`.
timeout.forget();
```

### Timeouts as `Future`s

```no_run
use futures::prelude::*;
use gloo_timers::TimeoutFuture;
use wasm_bindgen_futures::spawn_local;

let timeout = TimeoutFuture::new(1_000).and_then(|_| {
    // Do something here after the one second timeout is up!
#   Ok(())
});

// Spawn the `timeout` future on the local thread. If we just dropped it, then
// the timeout would be cancelled with `clearTimeout`.
spawn_local(timeout);
```

## Intervals

Intervals fire repeatedly every *n* milliseconds.

### Intervals with a Callback Function

TODO

### Intervals as `Stream`s

TODO

 */

#![deny(missing_docs, missing_debug_implementations)]

use futures::prelude::*;
use futures::sync::mpsc;
use std::fmt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

fn window() -> web_sys::Window {
    web_sys::window().unwrap_throw()
}

/// A scheduled timeout.
///
/// See `Timeout::new` for scheduling new timeouts.
///
/// Once scheduled, you can either `cancel` so that it doesn't run or `forget`
/// it so that it is un-cancel-able.
#[must_use = "timeouts cancel on drop; either call `forget` or `drop` explicitly"]
pub struct Timeout {
    id: Option<i32>,
    closure: Option<Closure<FnMut()>>,
}

impl Drop for Timeout {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            window().clear_timeout_with_handle(id);
        }
    }
}

impl fmt::Debug for Timeout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Timeout").field("id", &self.id).finish()
    }
}

impl Timeout {
    /// Schedule a timeout to invoke `callback` in `millis` milliseconds from
    /// now.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::Timeout;
    ///
    /// let timeout = Timeout::new(1_000, move || {
    ///     // Do something...
    /// });
    /// ```
    pub fn new<F>(millis: u32, callback: F) -> Timeout
    where
        F: 'static + FnOnce(),
    {
        // TODO: Use `FnOnce` here after this merges:
        // https://github.com/rustwasm/wasm-bindgen/pull/1281
        let mut callback = Some(callback);
        let closure = Closure::wrap(Box::new(move || {
            let callback = callback.take().unwrap_throw();
            callback();
        }) as Box<FnMut()>);

        let id = window()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref::<js_sys::Function>(),
                millis as i32,
            )
            .unwrap_throw();

        Timeout {
            id: Some(id),
            closure: Some(closure),
        }
    }

    /// Make this timeout uncancel-able.
    ///
    /// Returns the identifier returned by the original `setTimeout` call, and
    /// therefore you can still cancel the timeout by calling `clearTimeout`
    /// directly (perhaps via `web_sys::clear_timeout_with_handle`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::Timeout;
    ///
    /// // We definitely want to do stuff, and aren't going to ever cancel this
    /// // timeout.
    /// Timeout::new(1_000, || {
    ///     // Do stuff...
    /// }).forget();
    /// ```
    pub fn forget(mut self) -> i32 {
        let id = self.id.take().unwrap_throw();
        self.closure.take().unwrap_throw().forget();
        id
    }

    /// Cancel this timeout so that the callback is not invoked after the time
    /// is up.
    ///
    /// The scheduled callback is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::Timeout;
    ///
    /// let timeout = Timeout::new(1_000, || {
    ///     // Do stuff...
    /// });
    ///
    /// // If actually we didn't want to set a timer, then cancel it.
    /// if nevermind() {
    ///     timeout.cancel();
    /// }
    /// # fn nevermind() -> bool { true }
    /// ```
    pub fn cancel(mut self) -> Closure<FnMut()> {
        self.closure.take().unwrap_throw()
    }
}

/// A scheduled timeout as a `Future`.
///
/// See `TimeoutFuture::new` for scheduling new timeouts.
///
/// Once scheduled, if you change your mind and don't want the timeout to fire,
/// you can `drop` the future.
///
/// A timeout future will never resolve to `Err`. Its only failure mode is when
/// the timeout is so long that it is effectively infinite and never fires.
///
/// # Example
///
/// ```no_run
/// use futures::prelude::*;
/// use gloo_timers::TimeoutFuture;
///
/// let timeout_a = TimeoutFuture::new(1_000).map(|_| "a");
/// let timeout_b = TimeoutFuture::new(2_000).map(|_| "b");
///
/// wasm_bindgen_futures::spawn_local(
///     timeout_a
///         .select(timeout_b)
///         .and_then(|(who, other)| {
///             // `timeout_a` should have won this race.
///             assert_eq!(who, "a");
///
///             // Drop `timeout_b` to cancel its timeout.
///             drop(other);
///
///             Ok(())
///         })
///         .map_err(|_| {
///             wasm_bindgen::throw_str(
///                 "unreachable -- timeouts never fail, only potentially hang"
///             );
///         })
/// );
/// ```
#[must_use = "futures do nothing unless polled or spawned"]
pub struct TimeoutFuture {
    id: Option<i32>,
    inner: JsFuture,
}

impl Drop for TimeoutFuture {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            window().clear_timeout_with_handle(id);
        }
    }
}

impl TimeoutFuture {
    /// Create a new timeout future.
    ///
    /// Remember that futures do nothing unless polled or spawned, so either
    /// pass this future to `wasm_bindgen_futures::spawn_local` or use it inside
    /// another future.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::prelude::*;
    /// use gloo_timers::TimeoutFuture;
    ///
    /// wasm_bindgen_futures::spawn_local(
    ///     TimeoutFuture::new(1_000).map(|_| {
    ///         // Do stuff after one second...
    ///     })
    /// );
    /// ```
    pub fn new(millis: u32) -> TimeoutFuture {
        let mut id = None;
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            id = Some(
                window()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis as i32)
                    .unwrap_throw(),
            );
        });
        debug_assert!(id.is_some());
        let inner = JsFuture::from(promise);
        TimeoutFuture { id, inner }
    }
}

impl fmt::Debug for TimeoutFuture {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TimeoutFuture")
            .field("id", &self.id)
            .finish()
    }
}

impl Future for TimeoutFuture {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        match self.inner.poll() {
            Ok(Async::Ready(_)) => Ok(Async::Ready(())),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            // We only ever `resolve` the promise, never reject it.
            Err(_) => wasm_bindgen::throw_str("unreachable"),
        }
    }
}

/// A scheduled interval.
///
/// See `Interval::new` for scheduling new intervals.
///
/// Once scheduled, you can either `cancel` so that it ceases to fire or `forget`
/// it so that it is un-cancel-able.
#[must_use = "intervals cancel on drop; either call `forget` or `drop` explicitly"]
pub struct Interval {
    id: Option<i32>,
    closure: Option<Closure<FnMut()>>,
}

impl Drop for Interval {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            window().clear_interval_with_handle(id);
        }
    }
}

impl fmt::Debug for Interval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Interval").field("id", &self.id).finish()
    }
}

impl Interval {
    /// Schedule an interval to invoke `callback` every `millis` milliseconds.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::Interval;
    ///
    /// let interval = Interval::new(1_000, move || {
    ///     // Do something...
    /// });
    /// ```
    pub fn new<F>(millis: u32, callback: F) -> Interval
    where
        F: 'static + FnMut(),
    {
        let mut callback = Some(callback);
        let closure = Closure::wrap(Box::new(move || {
            let mut callback = callback.take().unwrap_throw();
            callback();
        }) as Box<FnMut()>);

        let id = window()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref::<js_sys::Function>(),
                millis as i32,
            )
            .unwrap_throw();

        Interval {
            id: Some(id),
            closure: Some(closure),
        }
    }

    /// Make this interval uncancel-able.
    ///
    /// Returns the identifier returned by the original `setInterval` call, and
    /// therefore you can still cancel the interval by calling `clearInterval`
    /// directly (perhaps via `web_sys::clear_interval_with_handle`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::Interval;
    ///
    /// // We want to do stuff every second, indefinitely.
    /// Interval::new(1_000, || {
    ///     // Do stuff...
    /// }).forget();
    /// ```
    pub fn forget(mut self) -> i32 {
        let id = self.id.take().unwrap_throw();
        self.closure.take().unwrap_throw().forget();
        id
    }

    /// Cancel this interval so that the callback is no longer periodically
    /// invoked.
    ///
    /// The scheduled callback is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::Interval;
    ///
    /// let interval = Interval::new(1_000, || {
    ///     // Do stuff...
    /// });
    ///
    /// // If we don't want this interval to run anymore, then cancel it.
    /// if nevermind() {
    ///     interval.cancel();
    /// }
    /// # fn nevermind() -> bool { true }
    /// ```
    pub fn cancel(mut self) -> Closure<FnMut()> {
        self.closure.take().unwrap_throw()
    }
}

/// A scheduled interval as a `Stream`.
///
/// See `IntervalStream::new` for scheduling new intervals.
///
/// Once scheduled, if you want to stop the interval from continuing to fire,
/// you can `drop` the stream.
///
/// An interval stream will never resolve to `Err`.
#[must_use = "streams do nothing unless polled or spawned"]
pub struct IntervalStream {
    millis: u32,
    id: Option<i32>,
    closure: Closure<FnMut()>,
    inner: mpsc::UnboundedReceiver<()>,
}

impl IntervalStream {
    /// Create a new interval stream.
    ///
    /// Remember that streams do nothing unless polled or spawned, so either
    /// spawn this stream via `wasm_bindgen_futures::spawn_local` or use it inside
    /// another stream or future.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use futures::prelude::*;
    /// use gloo_timers::IntervalStream;
    ///
    /// wasm_bindgen_futures::spawn_local(
    ///     IntervalStream::new(1_000)
    ///         .for_each(|_| {
    ///             // Do stuff every one second...
    ///             Ok(())
    ///         })
    /// );
    /// ```
    pub fn new(millis: u32) -> IntervalStream {
        let (sender, receiver) = mpsc::unbounded();
        let closure = Closure::wrap(Box::new(move || {
            sender.unbounded_send(()).unwrap();
        }) as Box<FnMut()>);

        IntervalStream {
            millis,
            id: None,
            closure,
            inner: receiver,
        }
    }
}

impl Drop for IntervalStream {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            window().clear_interval_with_handle(id);
        }
    }
}

impl fmt::Debug for IntervalStream {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("IntervalStream")
            .field("id", &self.id)
            .finish()
    }
}

impl Stream for IntervalStream {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Option<()>, ()> {
        if self.id.is_none() {
            self.id = Some(
                window()
                    .set_interval_with_callback_and_timeout_and_arguments_0(
                        self.closure.as_ref().unchecked_ref::<js_sys::Function>(),
                        self.millis as i32,
                    )
                    .unwrap_throw(),
            );
        }

        self.inner.poll()
    }
}
