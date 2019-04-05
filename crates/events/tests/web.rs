//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use futures::prelude::*;
use futures::sync::mpsc;
use gloo_events::{EventListener, EventListenerOptions, EventListenerPhase};
use js_sys::Error;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_test::*;
use web_sys::{window, HtmlElement};

wasm_bindgen_test_configure!(run_in_browser);

fn body() -> HtmlElement {
    window()
        .unwrap_throw()
        .document()
        .unwrap_throw()
        .body()
        .unwrap_throw()
}

fn is<A>(actual: A, expected: A) -> Result<(), JsValue>
where
    A: PartialEq + std::fmt::Debug,
{
    if expected == actual {
        Ok(())
    } else {
        Err(Error::new(&format!(
            "Expected:\n    {:#?}\nBut got:\n    {:#?}",
            expected, actual
        ))
        .into())
    }
}

#[derive(Clone)]
struct Sender<A> {
    sender: mpsc::UnboundedSender<Result<A, JsValue>>,
}

impl<A> Sender<A> {
    fn send<F>(&self, f: F)
    where
        F: FnOnce() -> Result<A, JsValue>,
    {
        self.sender.unbounded_send(f()).unwrap_throw();
    }
}

fn mpsc<A, F>(f: F) -> impl Future<Item = Vec<A>, Error = JsValue>
where
    F: FnOnce(Sender<A>),
{
    let (sender, receiver) = futures::sync::mpsc::unbounded();

    f(Sender { sender });

    receiver
        .then(|x| match x {
            Ok(a) => a,
            Err(_) => unreachable!(),
        })
        .collect()
}

// ----------------------------------------------------------------------------
// Tests begin here
// ----------------------------------------------------------------------------

#[wasm_bindgen_test(async)]
fn new_with_options() -> impl Future<Item = (), Error = JsValue> {
    mpsc(|sender| {
        let body = body();

        let _handler = EventListener::new_with_options(
            &body,
            "click",
            EventListenerOptions {
                phase: EventListenerPhase::Capture,
                passive: false,
            },
            move |e| {
                sender.send(|| {
                    is(e.dyn_into::<web_sys::MouseEvent>().is_ok(), true)?;

                    Ok(())
                });
            },
        );

        body.click();
        body.click();
    })
    .and_then(|results| is(results, vec![(), ()]))
}

#[wasm_bindgen_test(async)]
fn once_with_options() -> impl Future<Item = (), Error = JsValue> {
    mpsc(|sender| {
        let body = body();

        let _handler = EventListener::once_with_options(
            &body,
            "click",
            EventListenerOptions {
                phase: EventListenerPhase::Capture,
                passive: false,
            },
            move |e| {
                sender.send(|| {
                    is(e.dyn_into::<web_sys::MouseEvent>().is_ok(), true)?;

                    Ok(())
                });
            },
        );

        body.click();
        body.click();
    })
    .and_then(|results| is(results, vec![()]))
}

#[wasm_bindgen_test(async)]
fn new() -> impl Future<Item = (), Error = JsValue> {
    mpsc(|sender| {
        let body = body();

        let _handler = EventListener::new(&body, "click", move |e| {
            sender.send(|| {
                is(e.dyn_into::<web_sys::MouseEvent>().is_ok(), true)?;

                Ok(())
            });
        });

        body.click();
        body.click();
    })
    .and_then(|results| is(results, vec![(), ()]))
}

#[wasm_bindgen_test(async)]
fn once() -> impl Future<Item = (), Error = JsValue> {
    mpsc(|sender| {
        let body = body();

        let _handler = EventListener::once(&body, "click", move |e| {
            sender.send(|| {
                is(e.dyn_into::<web_sys::MouseEvent>().is_ok(), true)?;

                Ok(())
            });
        });

        body.click();
        body.click();
    })
    .and_then(|results| is(results, vec![()]))
}

// TODO is it possible to somehow cleanup the closure after a timeout?
#[wasm_bindgen_test]
fn forget() {
    let target = window()
        .unwrap_throw()
        .document()
        .unwrap_throw()
        .create_element("div")
        .unwrap_throw();

    let handler = EventListener::new(&target, "click", move |_| {});

    handler.forget();
}

#[wasm_bindgen_test(async)]
fn dynamic_registration() -> impl Future<Item = (), Error = JsValue> {
    mpsc(|sender| {
        let body = body();

        let handler1 = EventListener::new(&body, "click", {
            let sender = sender.clone();
            move |_| sender.send(|| Ok(1))
        });

        let handler2 = EventListener::new(&body, "click", {
            let sender = sender.clone();
            move |_| sender.send(|| Ok(2))
        });

        body.click();

        drop(handler1);

        body.click();

        let handler3 = EventListener::new(&body, "click", {
            let sender = sender.clone();
            move |_| sender.send(|| Ok(3))
        });

        body.click();

        drop(handler2);

        body.click();

        drop(handler3);

        body.click();
    })
    .and_then(|results| is(results, vec![1, 2, 2, 2, 3, 3]))
}
