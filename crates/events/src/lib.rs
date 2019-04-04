/*!
Using event listeners with [`web-sys`](https://crates.io/crates/web-sys) is hard! This crate provides an [`EventListener`](struct.EventListener.html) type which makes it easy!

See the documentation for [`EventListener`](struct.EventListener.html) for more information.
*/
#![deny(missing_docs, missing_debug_implementations)]

use std::borrow::Cow;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{AddEventListenerOptions, Event, EventTarget};

/// Specifies whether the event listener is run during the capture or bubble phase.
///
/// The official specification has [a good explanation](https://www.w3.org/TR/DOM-Level-3-Events/#event-flow)
/// of capturing vs bubbling.
///
/// # Default
///
/// ```rust
/// # use gloo_events::EventListenerPhase;
/// #
/// EventListenerPhase::Bubble
/// # ;
/// ```
#[derive(Debug, Clone, Copy)]
pub enum EventListenerPhase {
    #[allow(missing_docs)]
    Bubble,

    #[allow(missing_docs)]
    Capture,
}

impl EventListenerPhase {
    #[inline]
    fn is_capture(&self) -> bool {
        match self {
            EventListenerPhase::Bubble => false,
            EventListenerPhase::Capture => true,
        }
    }
}

impl Default for EventListenerPhase {
    #[inline]
    fn default() -> Self {
        EventListenerPhase::Bubble
    }
}

/// Specifies options for [`EventListener::new_with_options`](struct.EventListener.html#method.new_with_options) and
/// [`EventListener::once_with_options`](struct.EventListener.html#method.once_with_options).
///
/// # Default
///
/// ```rust
/// # use gloo_events::{EventListenerOptions, EventListenerPhase};
/// #
/// EventListenerOptions {
///     phase: EventListenerPhase::Bubble,
///     passive: true,
/// }
/// # ;
/// ```
///
/// # Examples
///
/// Sets `phase` to `EventListenerPhase::Capture`, using the default for the rest:
///
/// ```rust
/// # use gloo_events::EventListenerOptions;
/// #
/// let options = EventListenerOptions::run_in_capture_phase();
/// ```
///
/// Sets `passive` to `false`, using the default for the rest:
///
/// ```rust
/// # use gloo_events::EventListenerOptions;
/// #
/// let options = EventListenerOptions::enable_prevent_default();
/// ```
///
/// Specifies all options:
///
/// ```rust
/// # use gloo_events::{EventListenerOptions, EventListenerPhase};
/// #
/// let options = EventListenerOptions {
///     phase: EventListenerPhase::Capture,
///     passive: false,
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct EventListenerOptions {
    /// The phase that the event listener should be run in.
    pub phase: EventListenerPhase,

    /// If this is `true` then performance is improved, but it is not possible to use
    /// [`event.prevent_default()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Event.html#method.prevent_default).
    ///
    /// If this is `false` then performance might be reduced, but now it is possible to use
    /// [`event.prevent_default()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Event.html#method.prevent_default).
    ///
    /// You can read more about the performance costs
    /// [here](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#Improving_scrolling_performance_with_passive_listeners).
    pub passive: bool,
}

impl EventListenerOptions {
    /// Returns an `EventListenerOptions` with `phase` set to `EventListenerPhase::Capture`.
    ///
    /// This is the same as:
    ///
    /// ```rust
    /// # use gloo_events::{EventListenerOptions, EventListenerPhase};
    /// #
    /// EventListenerOptions {
    ///     phase: EventListenerPhase::Capture,
    ///     ..Default::default()
    /// }
    /// # ;
    /// ```
    #[inline]
    pub fn run_in_capture_phase() -> Self {
        Self {
            phase: EventListenerPhase::Capture,
            ..Self::default()
        }
    }

    /// Returns an `EventListenerOptions` with `passive` set to `false`.
    ///
    /// This is the same as:
    ///
    /// ```rust
    /// # use gloo_events::EventListenerOptions;
    /// #
    /// EventListenerOptions {
    ///     passive: false,
    ///     ..Default::default()
    /// }
    /// # ;
    /// ```
    #[inline]
    pub fn enable_prevent_default() -> Self {
        Self {
            passive: false,
            ..Self::default()
        }
    }

    #[inline]
    fn to_js(&self, once: bool) -> AddEventListenerOptions {
        let mut options = AddEventListenerOptions::new();

        options.capture(self.phase.is_capture());
        options.once(once);
        options.passive(self.passive);

        options
    }
}

impl Default for EventListenerOptions {
    #[inline]
    fn default() -> Self {
        Self {
            phase: Default::default(),
            passive: true,
        }
    }
}

// This defaults passive to true to avoid performance issues in browsers:
// https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#Improving_scrolling_performance_with_passive_listeners
thread_local! {
    static NEW_OPTIONS: AddEventListenerOptions = EventListenerOptions::default().to_js(false);
    static ONCE_OPTIONS: AddEventListenerOptions = EventListenerOptions::default().to_js(true);
}

/// RAII type which is used to manage DOM event listeners.
///
/// When the `EventListener` is dropped, it will automatically deregister the event listener and clean up the closure's memory.
///
/// Normally the `EventListener` is stored inside of another struct, like this:
///
/// ```rust
/// # use gloo_events::EventListener;
/// # use wasm_bindgen::UnwrapThrowExt;
/// use futures::Poll;
/// use futures::stream::Stream;
/// use futures::sync::mpsc;
/// use web_sys::EventTarget;
///
/// pub struct OnClick {
///     receiver: mpsc::UnboundedReceiver<()>,
///     // Automatically removed from the DOM on drop!
///     listener: EventListener,
/// }
///
/// impl OnClick {
///     pub fn new(target: &EventTarget) -> Self {
///         let (sender, receiver) = mpsc::unbounded();
///
///         // Attach an event listener
///         let listener = EventListener::new(&target, "click", move |_event| {
///             sender.unbounded_send(()).unwrap_throw();
///         });
///
///         Self {
///             receiver,
///             listener,
///         }
///     }
/// }
///
/// impl Stream for OnClick {
///     type Item = ();
///     type Error = ();
///
///     fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
///         self.receiver.poll().map_err(|_| unreachable!())
///     }
/// }
/// ```
#[must_use = "event listener will never be called after being dropped"]
pub struct EventListener {
    target: EventTarget,
    event_type: Cow<'static, str>,
    callback: Option<Closure<FnMut(Event)>>,
    phase: EventListenerPhase,
}

impl EventListener {
    #[inline]
    fn raw_new(
        target: &EventTarget,
        event_type: Cow<'static, str>,
        callback: Closure<FnMut(Event)>,
        options: &AddEventListenerOptions,
        phase: EventListenerPhase,
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
            phase,
        }
    }

    /// Registers an event listener on an [`EventTarget`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.EventTarget.html).
    ///
    /// For specifying options, there is a corresponding [`EventListener::new_with_options`](#method.new_with_options) method.
    ///
    /// If you only need the event to fire once, you can use [`EventListener::once`](#method.once) instead,
    /// which accepts an `FnOnce` closure.
    ///
    /// # Event type
    ///
    /// The event type can be either a `&'static str` like `"click"`, or it can be a dynamically constructed `String`.
    ///
    /// All event types are supported. Here is a [partial list](https://developer.mozilla.org/en-US/docs/Web/Events) of the available event types.
    ///
    /// # Passive
    ///
    /// [For performance reasons](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#Improving_scrolling_performance_with_passive_listeners),
    /// it is not possible to use [`event.prevent_default()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Event.html#method.prevent_default).
    ///
    /// If you need to use `prevent_default`, you must use [`EventListener::new_with_options`](#method.new_with_options), like this:
    ///
    /// ```rust,no_run
    /// # use gloo_events::{EventListener, EventListenerOptions};
    /// # let target = unimplemented!();
    /// # let event_type = "click";
    /// # let callback = move |e| {};
    /// #
    /// let options = EventListenerOptions::enable_prevent_default();
    ///
    /// EventListener::new_with_options(target, event_type, options, callback)
    /// # ;
    /// ```
    ///
    /// # Capture
    ///
    /// By default, event listeners are run in the bubble phase, *not* the capture phase. The official specification has
    /// [a good explanation](https://www.w3.org/TR/DOM-Level-3-Events/#event-flow) of capturing vs bubbling.
    ///
    /// If you want it to run in the capture phase, you must use [`EventListener::new_with_options`](#method.new_with_options), like this:
    ///
    /// ```rust,no_run
    /// # use gloo_events::{EventListener, EventListenerOptions};
    /// # let target = unimplemented!();
    /// # let event_type = "click";
    /// # let callback = move |e| {};
    /// #
    /// // This runs the event listener in the capture phase, rather than the bubble phase
    /// let options = EventListenerOptions::run_in_capture_phase();
    ///
    /// EventListener::new_with_options(target, event_type, options, callback)
    /// # ;
    /// ```
    ///
    /// # Examples
    ///
    /// Registers a [`"click"`](https://developer.mozilla.org/en-US/docs/Web/API/Element/click_event) event and downcasts it to the correct `Event` subtype
    /// (which is [`MouseEvent`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.MouseEvent.html)):
    ///
    /// ```rust,no_run
    /// # use gloo_events::EventListener;
    /// # use wasm_bindgen::{JsCast, UnwrapThrowExt};
    /// # let target = unimplemented!();
    /// #
    /// let listener = EventListener::new(&target, "click", move |event| {
    ///     let event = event.dyn_into::<web_sys::MouseEvent>().unwrap_throw();
    ///
    ///     // ...
    /// });
    /// ```
    #[inline]
    pub fn new<S, F>(target: &EventTarget, event_type: S, callback: F) -> Self
    where
        S: Into<Cow<'static, str>>,
        F: FnMut(Event) + 'static,
    {
        let callback = Closure::wrap(Box::new(callback) as Box<FnMut(Event)>);

        NEW_OPTIONS.with(move |options| {
            Self::raw_new(
                target,
                event_type.into(),
                callback,
                options,
                EventListenerPhase::Bubble,
            )
        })
    }

    /// This is exactly the same as [`EventListener::new`](#method.new), except the event will only fire once,
    /// and it accepts `FnOnce` instead of `FnMut`.
    ///
    /// For specifying options, there is a corresponding [`EventListener::once_with_options`](#method.once_with_options) method.
    ///
    /// # Examples
    ///
    /// Registers a [`"load"`](https://developer.mozilla.org/en-US/docs/Web/API/FileReader/load_event) event and casts it to the correct type
    /// (which is [`ProgressEvent`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.ProgressEvent.html)):
    ///
    /// ```rust,no_run
    /// # use gloo_events::EventListener;
    /// # use wasm_bindgen::{JsCast, UnwrapThrowExt};
    /// # let target = unimplemented!();
    /// #
    /// let listener = EventListener::once(&target, "load", move |event| {
    ///     let event = event.dyn_into::<web_sys::ProgressEvent>().unwrap_throw();
    ///
    ///     // ...
    /// });
    /// ```
    #[inline]
    pub fn once<S, F>(target: &EventTarget, event_type: S, callback: F) -> Self
    where
        S: Into<Cow<'static, str>>,
        F: FnOnce(Event) + 'static,
    {
        let callback = Closure::once(callback);

        ONCE_OPTIONS.with(move |options| {
            Self::raw_new(
                target,
                event_type.into(),
                callback,
                options,
                EventListenerPhase::Bubble,
            )
        })
    }

    /// Registers an event listener on an [`EventTarget`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.EventTarget.html).
    ///
    /// It is recommended to use [`EventListener::new`](#method.new) instead, because it has better performance, and it is more convenient.
    ///
    /// If you only need the event to fire once, you can use [`EventListener::once_with_options`](#method.once_with_options) instead,
    /// which accepts an `FnOnce` closure.
    ///
    /// # Event type
    ///
    /// The event type can be either a `&'static str` like `"click"`, or it can be a dynamically constructed `String`.
    ///
    /// All event types are supported. Here is a [partial list](https://developer.mozilla.org/en-US/docs/Web/Events)
    /// of the available event types.
    ///
    /// # Options
    ///
    /// See the documentation for [`EventListenerOptions`](struct.EventListenerOptions.html) for more details.
    ///
    /// # Examples
    ///
    /// Registers a [`"touchstart"`](https://developer.mozilla.org/en-US/docs/Web/API/Element/touchstart_event)
    /// event and uses
    /// [`event.prevent_default()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Event.html#method.prevent_default):
    ///
    /// ```rust,no_run
    /// # use gloo_events::{EventListener, EventListenerOptions};
    /// # let target = unimplemented!();
    /// #
    /// let options = EventListenerOptions::enable_prevent_default();
    ///
    /// let listener = EventListener::new_with_options(&target, "touchstart", options, move |event| {
    ///     event.prevent_default();
    ///
    ///     // ...
    /// });
    /// ```
    ///
    /// Registers a [`"click"`](https://developer.mozilla.org/en-US/docs/Web/API/Element/click_event)
    /// event in the capturing phase and uses
    /// [`event.stop_propagation()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Event.html#method.stop_propagation)
    /// to stop the event from bubbling:
    ///
    /// ```rust,no_run
    /// # use gloo_events::{EventListener, EventListenerOptions};
    /// # let target = unimplemented!();
    /// #
    /// let options = EventListenerOptions::run_in_capture_phase();
    ///
    /// let listener = EventListener::new_with_options(&target, "click", options, move |event| {
    ///     // Stop the event from bubbling
    ///     event.stop_propagation();
    ///
    ///     // ...
    /// });
    /// ```
    #[inline]
    pub fn new_with_options<S, F>(
        target: &EventTarget,
        event_type: S,
        options: EventListenerOptions,
        callback: F,
    ) -> Self
    where
        S: Into<Cow<'static, str>>,
        F: FnMut(Event) + 'static,
    {
        let callback = Closure::wrap(Box::new(callback) as Box<FnMut(Event)>);

        Self::raw_new(
            target,
            event_type.into(),
            callback,
            &options.to_js(false),
            options.phase,
        )
    }

    /// This is exactly the same as [`EventListener::new_with_options`](#method.new_with_options), except the event will only fire once,
    /// and it accepts `FnOnce` instead of `FnMut`.
    ///
    /// It is recommended to use [`EventListener::once`](#method.once) instead, because it has better performance, and it is more convenient.
    ///
    /// # Examples
    ///
    /// Registers a [`"load"`](https://developer.mozilla.org/en-US/docs/Web/API/FileReader/load_event)
    /// event and uses
    /// [`event.prevent_default()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Event.html#method.prevent_default):
    ///
    /// ```rust,no_run
    /// # use gloo_events::{EventListener, EventListenerOptions};
    /// # let target = unimplemented!();
    /// #
    /// let options = EventListenerOptions::enable_prevent_default();
    ///
    /// let listener = EventListener::once_with_options(&target, "load", options, move |event| {
    ///     event.prevent_default();
    ///
    ///     // ...
    /// });
    /// ```
    ///
    /// Registers a [`"click"`](https://developer.mozilla.org/en-US/docs/Web/API/Element/click_event)
    /// event in the capturing phase and uses
    /// [`event.stop_propagation()`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Event.html#method.stop_propagation)
    /// to stop the event from bubbling:
    ///
    /// ```rust,no_run
    /// # use gloo_events::{EventListener, EventListenerOptions};
    /// # let target = unimplemented!();
    /// #
    /// let options = EventListenerOptions::run_in_capture_phase();
    ///
    /// let listener = EventListener::once_with_options(&target, "click", options, move |event| {
    ///     // Stop the event from bubbling
    ///     event.stop_propagation();
    ///
    ///     // ...
    /// });
    /// ```
    #[inline]
    pub fn once_with_options<S, F>(
        target: &EventTarget,
        event_type: S,
        options: EventListenerOptions,
        callback: F,
    ) -> Self
    where
        S: Into<Cow<'static, str>>,
        F: FnOnce(Event) + 'static,
    {
        let callback = Closure::once(callback);

        Self::raw_new(
            target,
            event_type.into(),
            callback,
            &options.to_js(true),
            options.phase,
        )
    }

    /// Keeps the `EventListener` alive forever, so it will never be dropped.
    ///
    /// This should only be used when you want the `EventListener` to last forever, otherwise it will leak memory!
    #[inline]
    pub fn forget(mut self) {
        // take() is necessary because of Rust's restrictions about Drop
        // This will never panic, because `callback` is always `Some`
        self.callback.take().unwrap_throw().forget()
    }

    /// Returns the `EventTarget`.
    #[inline]
    pub fn target(&self) -> &EventTarget {
        &self.target
    }

    /// Returns the event type.
    #[inline]
    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    /// Returns the callback.
    #[inline]
    pub fn callback(&self) -> &Closure<FnMut(Event)> {
        // This will never panic, because `callback` is always `Some`
        self.callback.as_ref().unwrap_throw()
    }

    /// Returns whether the event listener is run during the capture or bubble phase.
    ///
    /// The official specification has [a good explanation](https://www.w3.org/TR/DOM-Level-3-Events/#event-flow)
    /// of capturing vs bubbling.
    #[inline]
    pub fn phase(&self) -> EventListenerPhase {
        self.phase
    }
}

impl Drop for EventListener {
    #[inline]
    fn drop(&mut self) {
        // This will only be None if forget() was called
        if let Some(callback) = &self.callback {
            self.target
                .remove_event_listener_with_callback_and_bool(
                    self.event_type(),
                    callback.as_ref().unchecked_ref(),
                    self.phase.is_capture(),
                )
                .unwrap_throw();
        }
    }
}

// TODO Remove this after https://github.com/rustwasm/wasm-bindgen/issues/1387 is fixed
impl std::fmt::Debug for EventListener {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EventListener")
            .field("target", &self.target)
            .field("event_type", &self.event_type)
            .field("callback", &"Closure { ... }")
            .field("phase", &self.phase)
            .finish()
    }
}
