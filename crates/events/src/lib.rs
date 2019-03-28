/*!
 */
#![deny(missing_docs, missing_debug_implementations)]

use std::borrow::Cow;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{AddEventListenerOptions, Event, EventTarget};

///
#[derive(Debug, Clone, Copy)]
pub struct EventListenerOptions {
    ///
    pub capture: bool,

    ///
    pub once: bool,

    ///
    pub passive: bool,
}

impl EventListenerOptions {
    ///
    #[inline]
    pub fn to_js(&self) -> AddEventListenerOptions {
        let mut options = AddEventListenerOptions::new();

        options.capture(self.capture);
        options.once(self.once);
        options.passive(self.passive);

        options
    }
}

// This defaults passive to true to avoid performance issues in browsers:
// https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#Improving_scrolling_performance_with_passive_listeners
thread_local! {
    static NEW_OPTIONS: AddEventListenerOptions = EventListenerOptions {
        capture: false,
        once: false,
        passive: true,
    }.to_js();

    static ONCE_OPTIONS: AddEventListenerOptions = EventListenerOptions {
        capture: false,
        once: true,
        passive: true,
    }.to_js();
}

///
pub struct EventListener {
    target: EventTarget,
    event_type: Cow<'static, str>,
    callback: Option<Closure<FnMut(Event)>>,
    is_capture: bool,
}

impl EventListener {
    #[inline]
    fn new_(
        target: &EventTarget,
        event_type: Cow<'static, str>,
        callback: Closure<FnMut(Event)>,
        options: &AddEventListenerOptions,
        is_capture: bool,
    ) -> Self {
        target
            .add_event_listener_with_callback_and_add_event_listener_options(
                &event_type,
                callback.as_ref().unchecked_ref(),
                options,
            )
            .unwrap_throw();

        Self {
            target: target.clone(),
            event_type,
            callback: Some(callback),
            is_capture,
        }
    }

    ///
    #[inline]
    pub fn new_with_options<A, F>(
        target: &EventTarget,
        event_type: A,
        options: &EventListenerOptions,
        callback: F,
    ) -> Self
    where
        A: Into<Cow<'static, str>>,
        F: FnMut(Event) + 'static,
    {
        let callback = Closure::wrap(Box::new(callback) as Box<FnMut(Event)>);

        Self::new_(
            target,
            event_type.into(),
            callback,
            &options.to_js(),
            options.capture,
        )
    }

    ///
    #[inline]
    pub fn new<A, F>(target: &EventTarget, event_type: A, callback: F) -> Self
    where
        A: Into<Cow<'static, str>>,
        F: FnMut(Event) + 'static,
    {
        let callback = Closure::wrap(Box::new(callback) as Box<FnMut(Event)>);

        NEW_OPTIONS
            .with(move |options| Self::new_(target, event_type.into(), callback, options, false))
    }

    ///
    #[inline]
    pub fn once<A, F>(target: &EventTarget, event_type: A, callback: F) -> Self
    where
        A: Into<Cow<'static, str>>,
        F: FnOnce(Event) + 'static,
    {
        let callback = Closure::once(callback);

        ONCE_OPTIONS
            .with(move |options| Self::new_(target, event_type.into(), callback, options, false))
    }

    ///
    #[inline]
    pub fn forget(mut self) {
        // take() is necessary because of Rust's restrictions about Drop
        // This will never panic, because `callback` is always `Some`
        self.callback.take().unwrap_throw().forget()
    }

    ///
    #[inline]
    pub fn target(&self) -> &EventTarget {
        &self.target
    }

    ///
    #[inline]
    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    ///
    #[inline]
    pub fn callback(&self) -> &Closure<FnMut(Event)> {
        // This will never panic, because `callback` is always `Some`
        self.callback.as_ref().unwrap_throw()
    }

    ///
    #[inline]
    pub fn is_capture(&self) -> bool {
        self.is_capture
    }
}

impl Drop for EventListener {
    #[inline]
    fn drop(&mut self) {
        self.target
            .remove_event_listener_with_callback_and_bool(
                self.event_type(),
                self.callback().as_ref().unchecked_ref(),
                self.is_capture(),
            )
            .unwrap_throw();
    }
}

// TODO Remove this after https://github.com/rustwasm/wasm-bindgen/issues/1387 is fixed
impl std::fmt::Debug for EventListener {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EventListener")
            .field("target", &self.target)
            .field("event_type", &self.event_type)
            .field("callback", &"Closure { ... }")
            .field("is_capture", &self.is_capture)
            .finish()
    }
}
