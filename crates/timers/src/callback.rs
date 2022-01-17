//! Callback-style timer APIs.

use js_sys::Function;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Window, WorkerGlobalScope};

thread_local! {
    static GLOBAL: WindowOrWorkerOrNode = WindowOrWorkerOrNode::new();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "setTimeout", catch)]
    fn set_timeout_with_callback_and_timeout_and_arguments_0(
        handler: &Function,
        timeout: i32,
    ) -> Result<i32, JsValue>;

    #[wasm_bindgen(js_name = "setInterval", catch)]
    fn set_interval_with_callback_and_timeout_and_arguments_0(
        handler: &Function,
        timeout: i32,
    ) -> Result<i32, JsValue>;

    #[wasm_bindgen(js_name = "clearTimeout")]
    fn clear_timeout_with_handle(handle: i32);

    #[wasm_bindgen(js_name = "clearInterval")]
    fn clear_interval_with_handle(handle: i32);
}

enum WindowOrWorkerOrNode {
    Window(Window),
    Worker(WorkerGlobalScope),
    NodeJS,
}

impl WindowOrWorkerOrNode {
    fn new() -> Self {
        #[wasm_bindgen]
        extern "C" {
            type Global;

            #[wasm_bindgen(method, getter, js_name = Window)]
            fn window(this: &Global) -> JsValue;

            #[wasm_bindgen(method, getter, js_name = WorkerGlobalScope)]
            fn worker(this: &Global) -> JsValue;

            #[wasm_bindgen(method, getter)]
            fn process(this: &Global) -> JsValue;
        }

        let global: Global = js_sys::global().unchecked_into();

        if !global.window().is_undefined() {
            Self::Window(global.unchecked_into())
        } else if !global.worker().is_undefined() {
            Self::Worker(global.unchecked_into())
        } else if !global.process().is_undefined() {
            Self::NodeJS
        } else {
            panic!("Only supported in a browser, web worker or node.js");
        }
    }
}

macro_rules! impl_window_or_worker {
    ($(fn $name:ident($($par_name:ident: $par_type:ty),*)$( -> $return:ty)?;)+) => {
        impl WindowOrWorkerOrNode {
            $(
                fn $name(&self, $($par_name: $par_type),*)$( -> $return)? {
                    match self {
                        Self::Window(window) => window.$name($($par_name),*),
                        Self::Worker(worker) => worker.$name($($par_name),*),
                        Self::NodeJS => $name($($par_name),*),
                    }
                }
            )+
        }
    };
}

impl_window_or_worker! {
    fn set_timeout_with_callback_and_timeout_and_arguments_0(handler: &Function, timeout: i32) -> Result<i32, JsValue>;
    fn set_interval_with_callback_and_timeout_and_arguments_0(handler: &Function, timeout: i32) -> Result<i32, JsValue>;
    fn clear_timeout_with_handle(handle: i32);
    fn clear_interval_with_handle(handle: i32);
}

/// A scheduled timeout.
///
/// See `Timeout::new` for scheduling new timeouts.
///
/// Once scheduled, you can either `cancel` so that it doesn't run or `forget`
/// it so that it is un-cancel-able.
#[derive(Debug)]
#[must_use = "timeouts cancel on drop; either call `forget` or `drop` explicitly"]
pub struct Timeout {
    id: Option<i32>,
    closure: Option<Closure<dyn FnMut()>>,
}

impl Drop for Timeout {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            GLOBAL.with(|global| global.clear_timeout_with_handle(id));
        }
    }
}

impl Timeout {
    /// Schedule a timeout to invoke `callback` in `millis` milliseconds from
    /// now.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::callback::Timeout;
    ///
    /// let timeout = Timeout::new(1_000, move || {
    ///     // Do something...
    /// });
    /// ```
    pub fn new<F>(millis: u32, callback: F) -> Timeout
    where
        F: 'static + FnOnce(),
    {
        let closure = Closure::once(callback);

        let id = GLOBAL.with(|global| {
            global
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref::<js_sys::Function>(),
                    millis as i32,
                )
                .unwrap_throw()
        });

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
    /// use gloo_timers::callback::Timeout;
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
    /// use gloo_timers::callback::Timeout;
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
    pub fn cancel(mut self) -> Closure<dyn FnMut()> {
        self.closure.take().unwrap_throw()
    }
}

/// A scheduled interval.
///
/// See `Interval::new` for scheduling new intervals.
///
/// Once scheduled, you can either `cancel` so that it ceases to fire or `forget`
/// it so that it is un-cancel-able.
#[derive(Debug)]
#[must_use = "intervals cancel on drop; either call `forget` or `drop` explicitly"]
pub struct Interval {
    id: Option<i32>,
    closure: Option<Closure<dyn FnMut()>>,
}

impl Drop for Interval {
    fn drop(&mut self) {
        if let Some(id) = self.id {
            GLOBAL.with(|global| global.clear_interval_with_handle(id));
        }
    }
}

impl Interval {
    /// Schedule an interval to invoke `callback` every `millis` milliseconds.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use gloo_timers::callback::Interval;
    ///
    /// let interval = Interval::new(1_000, move || {
    ///     // Do something...
    /// });
    /// ```
    pub fn new<F>(millis: u32, callback: F) -> Interval
    where
        F: 'static + FnMut(),
    {
        let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);

        let id = GLOBAL.with(|global| {
            global
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref::<js_sys::Function>(),
                    millis as i32,
                )
                .unwrap_throw()
        });

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
    /// use gloo_timers::callback::Interval;
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
    /// use gloo_timers::callback::Interval;
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
    pub fn cancel(mut self) -> Closure<dyn FnMut()> {
        self.closure.take().unwrap_throw()
    }
}
