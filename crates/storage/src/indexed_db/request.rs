//! Contains wrappers for `IDBRequest`. These are only exposed to the user as either a `Future` or
//! a `Stream`.
use futures::stream::Stream;
use gloo_events::{EventListener, EventListenerOptions};
use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};
use wasm_bindgen::prelude::*;
use web_sys::{DomException, IdbOpenDbRequest, IdbRequest, IdbRequestReadyState};

use crate::indexed_db::util::{unreachable_throw, UnreachableExt};

/// Wrapper around IdbRequest that implements `Future`.
pub(crate) struct Request {
    inner: IdbRequest,
    bubble_errors: bool,
    success_listener: Option<EventListener>,
    error_listener: Option<EventListener>,
}

impl Request {
    pub(crate) fn new(inner: IdbRequest, bubble_errors: bool) -> Self {
        Self {
            inner,
            bubble_errors,
            success_listener: None,
            error_listener: None,
        }
    }
}

impl Future for Request {
    type Output = Result<JsValue, DomException>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.ready_state() {
            IdbRequestReadyState::Pending => {
                if self.success_listener.is_none() {
                    self.success_listener = Some(EventListener::once(&self.inner, "success", {
                        let waker = cx.waker().clone();
                        move |_| waker.wake()
                    }))
                }
                if self.error_listener.is_none() {
                    let opts = if self.bubble_errors {
                        EventListenerOptions::enable_prevent_default()
                    } else {
                        EventListenerOptions::default()
                    };
                    self.error_listener = Some(EventListener::once_with_options(
                        &self.inner,
                        "error",
                        opts,
                        {
                            let waker = cx.waker().clone();
                            let bubble_errors = self.bubble_errors;
                            move |event| {
                                waker.wake();
                                if !bubble_errors {
                                    event.prevent_default();
                                }
                            }
                        },
                    ))
                }
                Poll::Pending
            }
            IdbRequestReadyState::Done => {
                if let Some(error) = self.inner.error().unreachable_throw() {
                    Poll::Ready(Err(error))
                } else {
                    // no error = success
                    Poll::Ready(Ok(self.inner.result().unreachable_throw()))
                }
            }
            _ => unreachable_throw(),
        }
    }
}

/// Special `IDBRequest` wrapper that optionally handles the `blocked` event, returning an error if
/// the request would block on another user operation.
///
/// Users can set the error on block flag if concurrent use of the database indicates an error.
pub(crate) struct OpenDbRequest {
    inner: Request,
    error_on_block: bool,
    blocked_listener: Option<EventListener>,
    blocked: Arc<AtomicBool>,
}

impl OpenDbRequest {
    pub(crate) fn new(inner: IdbOpenDbRequest, error_on_block: bool) -> Self {
        Self {
            inner: Request::new(inner.into(), true),
            error_on_block,
            blocked_listener: None,
            blocked: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Future for OpenDbRequest {
    type Output = Result<JsValue, DomException>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.blocked.load(Ordering::SeqCst) {
            // return error
            return Poll::Ready(Err(DomException::new_with_message_and_name(
                "transaction would block",
                "TransactionWouldBlock",
            )
            .unreachable_throw()));
        }

        match Pin::new(&mut self.inner).poll(cx) {
            Poll::Pending => {
                if self.error_on_block {
                    if self.blocked_listener.is_none() {
                        self.blocked_listener =
                            Some(EventListener::once(&self.inner.inner, "blocked", {
                                let blocked = self.blocked.clone();
                                let waker = cx.waker().clone();
                                move |_| {
                                    blocked.store(true, Ordering::SeqCst);
                                    waker.wake();
                                }
                            }))
                    }
                }
                Poll::Pending
            }
            ready => ready,
        }
    }
}

/// Wrapper for IDBRequest where the success callback is run multiple times.
// TODO If a task is woken up, does `wasm_bindgen_futures` try to progress the future in the same
// microtask or a separate one? This will impact whether I need to have space for more than one
// result at a time.
#[derive(Debug)]
pub(crate) struct StreamingRequest {
    inner: IdbRequest,
    bubble_errors: bool,
    success_listener: Option<EventListener>,
    error_listener: Option<EventListener>,
}

impl StreamingRequest {
    pub(crate) fn new(inner: IdbRequest, bubble_errors: bool) -> Self {
        Self {
            inner,
            bubble_errors,
            success_listener: None,
            error_listener: None,
        }
    }
}

impl Stream for StreamingRequest {
    type Item = Result<JsValue, DomException>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.ready_state() {
            IdbRequestReadyState::Pending => {
                if self.success_listener.is_none() {
                    // First call - setup
                    self.success_listener = Some(EventListener::new(&self.inner, "success", {
                        let waker = cx.waker().clone();
                        move |_| {
                            let waker = waker.clone();
                            waker.wake()
                        }
                    }));

                    // omit the error.is_none check to save a branch.
                    let opts = if self.bubble_errors {
                        EventListenerOptions::enable_prevent_default()
                    } else {
                        EventListenerOptions::default()
                    };
                    self.error_listener = Some(EventListener::new_with_options(
                        &self.inner,
                        "error",
                        opts,
                        {
                            let waker = cx.waker().clone();
                            let bubble_errors = self.bubble_errors;
                            move |event| {
                                let waker = waker.clone();
                                waker.wake();
                                if !bubble_errors {
                                    event.prevent_default();
                                }
                            }
                        },
                    ));
                }

                Poll::Pending
            }
            IdbRequestReadyState::Done => {
                if let Some(error) = self.inner.error().unreachable_throw() {
                    Poll::Ready(Some(Err(error)))
                } else {
                    // no error = success
                    // if the result is null, there won't be any more entries (at least for
                    // IDBCursor, which I think is the only case a request is re-used)
                    let result = self.inner.result().unreachable_throw();
                    if result.is_null() || result.is_undefined() {
                        Poll::Ready(None)
                    } else {
                        Poll::Ready(Some(Ok(result)))
                    }
                }
            }
            _ => unreachable_throw(),
        }
    }
}
