//! `Future`- and `Stream`-backed timers APIs.

use super::sys::*;
use crate::callback::Timeout;

use futures_channel::mpsc;
use futures_core::stream::Stream;
use std::future::Future;
use std::pin::Pin;
use std::task::{Poll, Context, Waker};
use std::sync::{Arc, Mutex};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

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
/// use gloo_timers::future::TimeoutFuture;
/// use futures_util::future::{select, Either};
/// use wasm_bindgen_futures::spawn_local;
///
/// spawn_local(async {
///     match select(TimeoutFuture::new(1_000), TimeoutFuture::new(2_000)).await {
///         Either::Left((val, b)) => {
///             // Drop the `2_000` ms timeout to cancel its timeout.
///             drop(b);
///         }
///         Either::Right((a, val)) => {
///             panic!("the `1_000` ms timeout should have won this race");
///         }
///     }
/// });
/// ```
#[derive(Debug)]
#[must_use = "futures do nothing unless polled or spawned"]
pub struct TimeoutFuture {
    inner: Timeout,
    state: Arc<Mutex<TimeoutFutureState>>,
}

/// A state machine for the timeout future.
#[derive(Debug)]
enum TimeoutFutureState {
    Init,
    Polled(Waker),
    Complete,
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
    /// use gloo_timers::future::TimeoutFuture;
    /// use wasm_bindgen_futures::spawn_local;
    ///
    /// spawn_local(async {
    ///     TimeoutFuture::new(1_000).await;
    ///     // Do stuff after one second...
    /// });
    /// ```
    pub fn new(millis: u32) -> TimeoutFuture {
        let state = Arc::new(Mutex::new(TimeoutFutureState::Init));
        let state_ref = Arc::downgrade(&state);
        let inner = Timeout::new(millis, move || {
            let state = match state_ref.upgrade() {
                Some(s) => s,
                None => return
            };
            let mut state = state.lock().expect("mutex should not be poisoned");
            match &*state {
                TimeoutFutureState::Polled(waker) => {
                    waker.wake_by_ref();
                }
                _ => ()
            }
            (*state) = TimeoutFutureState::Complete;
        });
        TimeoutFuture { inner, state }
    }
}

impl Future for TimeoutFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        match *state {
            TimeoutFutureState::Init | TimeoutFutureState::Polled(_) => {
                (*state) = TimeoutFutureState::Polled(cx.waker().clone());
                Poll::Pending
            }
            TimeoutFutureState::Complete => Poll::Ready(()),
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
    /// use gloo_timers::future::IntervalStream;
    /// use futures_util::stream::StreamExt;
    /// use wasm_bindgen_futures::spawn_local;
    ///
    /// spawn_local(async {
    ///     IntervalStream::new(1_000).for_each(|_| {
    ///         // Do stuff every one second...
    ///     }).await;
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

        Pin::new(&mut self.inner).poll_next(cx)
    }
}
