/*!
 */
#![deny(missing_docs, missing_debug_implementations)]

use wasm_bindgen::{JsCast, UnwrapThrowExt};
use wasm_bindgen::closure::Closure;
use web_sys::{Event, EventTarget};


///
pub struct EventListener<'a> {
    target: EventTarget,
    kind: &'a str,
    callback: Option<Closure<FnMut(Event)>>,
}

impl<'a> EventListener<'a> {
    #[inline]
    fn new_closure(target: &EventTarget, kind: &'a str, callback: Closure<FnMut(Event)>) -> Self {
        target.add_event_listener_with_callback(kind, callback.as_ref().unchecked_ref()).unwrap_throw();

        Self {
            target: target.clone(),
            kind,
            callback: Some(callback),
        }
    }

    ///
    #[inline]
    pub fn new<F>(target: &EventTarget, kind: &'a str, callback: F) -> Self where F: FnMut(Event) + 'static {
        let callback = Closure::wrap(Box::new(callback) as Box<FnMut(Event)>);
        Self::new_closure(target, kind, callback)
    }

    ///
    #[inline]
    pub fn once<F>(target: &EventTarget, kind: &'a str, callback: F) -> Self where F: FnOnce(Event) + 'static {
        let callback = Closure::once(callback);
        Self::new_closure(target, kind, callback)
    }

    ///
    #[inline]
    pub fn forget(mut self) {
        // take() is necessary because of Rust's restrictions about Drop
        self.callback.take().unwrap_throw().forget()
    }
}

impl<'a> Drop for EventListener<'a> {
    #[inline]
    fn drop(&mut self) {
        self.target.remove_event_listener_with_callback(self.kind, self.callback.as_ref().unwrap_throw().as_ref().unchecked_ref()).unwrap_throw();
    }
}

impl<'a> std::fmt::Debug for EventListener<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EventListener")
            .field("target", &self.target)
            .field("kind", &self.kind)
            .field("callback", &"Closure { ... }")
            .finish()
    }
}
