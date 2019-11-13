//! `Future`- and `Stream`-backed timers APIs.

use std::task::{Poll, Context};
use std::pin::Pin;
use std::future::Future;
use super::sys::*;
use futures::stream::Stream;
use futures::channel::mpsc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

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
/// # extern crate futures_rs as futures;
/// use futures::prelude::*;
/// use gloo_timers::future::TimeoutFuture;
///
/// let timeout_a = TimeoutFuture::new(1_000);
/// let timeout_b = TimeoutFuture::new(2_000);
///
/// wasm_bindgen_futures::spawn_local(async {
///     match future::select(timeout_a, timeout_b).await {
///         future::Either::Left((val, b)) => {
///             // Drop `timeout_b` to cancel its timeout.
///             drop(b);
///         }
///         future::Either::Right((a, val)) => {
///             panic!("timeout_a should have won this race")
///         }
///     }
/// });
/// ```
#[derive(Debug)]
#[must_use = "futures do nothing unless polled or spawned"]
pub struct TimeoutFuture {
    id: Option<i32>,
    inner: JsFuture,
}

impl Drop for TimeoutFuture {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            clear_timeout(id);
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
    /// # extern crate futures_rs as futures;
    /// use futures::prelude::*;
    /// use gloo_timers::future::TimeoutFuture;
    ///
    /// wasm_bindgen_futures::spawn_local(async {
    ///     TimeoutFuture::new(1_000).await;
    ///     // Do stuff after one second...
    /// });
    /// ```
    pub fn new(millis: u32) -> TimeoutFuture {
        let mut id = None;
        let promise = js_sys::Promise::new(&mut |resolve, _reject| {
            id = Some(set_timeout(&resolve, millis as i32));
        });
        debug_assert!(id.is_some());
        let inner = JsFuture::from(promise);
        TimeoutFuture { id, inner }
    }

    /// Project the pin onto the inner future.
    fn project_inner(self: Pin<&mut Self>) -> Pin<&mut JsFuture> {
        // Unsafe: The field `inner` must not be borrowed without structural pinning.
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }
    }
}

impl Future for TimeoutFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match self.project_inner().poll(cx) {
            Poll::Ready(result) => {
                // We only ever `resolve` the promise, never reject it.
                debug_assert!(result.is_ok());
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending
        }
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
#[derive(Debug)]
#[must_use = "streams do nothing unless polled or spawned"]
pub struct IntervalStream {
    millis: u32,
    id: Option<i32>,
    closure: Closure<dyn FnMut()>,
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
    /// # extern crate futures_rs as futures;
    /// use futures::prelude::*;
    /// use gloo_timers::future::IntervalStream;
    ///
    /// wasm_bindgen_futures::spawn_local(async {
    ///     let mut interval = IntervalStream::new(1_000);
    ///     loop {
    ///         interval.next().await;
    ///         // Do stuff every one second...
    ///     }
    /// });
    /// ```
    pub fn new(millis: u32) -> IntervalStream {
        let (sender, receiver) = mpsc::unbounded();
        let closure = Closure::wrap(Box::new(move || {
            sender.unbounded_send(()).unwrap();
        }) as Box<dyn FnMut()>);

        IntervalStream {
            millis,
            id: None,
            closure,
            inner: receiver,
        }
    }

    /// Project the pin onto the inner stream
    fn project_inner(self: Pin<&mut Self>) -> Pin<&mut mpsc::UnboundedReceiver<()>> {
        // Unsafe: The field `inner` must not be borrowed without structural pinning.
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }
    }
}

impl Drop for IntervalStream {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            clear_interval(id);
        }
    }
}

impl Stream for IntervalStream {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        if self.id.is_none() {
            self.id = Some(set_interval(
                self.closure.as_ref().unchecked_ref::<js_sys::Function>(),
                self.millis as i32,
            ));
        }

        self.project_inner().poll_next(cx)
    }
}
