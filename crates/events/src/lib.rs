/*!
 */
#![deny(missing_docs, missing_debug_implementations)]

use wasm_bindgen::{JsCast, UnwrapThrowExt};
use wasm_bindgen::closure::Closure;
use web_sys::{Event, EventTarget};


///
pub struct EventListener<'a> {
    target: EventTarget,
    event_type: &'a str,
    callback: Option<Closure<FnMut(Event)>>,
}

impl<'a> EventListener<'a> {
    #[inline]
    fn new_closure(target: &EventTarget, event_type: &'a str, callback: Closure<FnMut(Event)>) -> Self {
        target.add_event_listener_with_callback(event_type, callback.as_ref().unchecked_ref()).unwrap_throw();

        Self {
            target: target.clone(),
            event_type,
            callback: Some(callback),
        }
    }

    ///
    #[inline]
    pub fn new<F>(target: &EventTarget, event_type: &'a str, callback: F) -> Self where F: FnMut(Event) + 'static {
        let callback = Closure::wrap(Box::new(callback) as Box<FnMut(Event)>);
        Self::new_closure(target, event_type, callback)
    }

    ///
    #[inline]
    pub fn once<F>(target: &EventTarget, event_type: &'a str, callback: F) -> Self where F: FnOnce(Event) + 'static {
        let callback = Closure::once(callback);
        Self::new_closure(target, event_type, callback)
    }

    ///
    #[inline]
    pub fn forget(mut self) {
        // take() is necessary because of Rust's restrictions about Drop
        self.callback.take().unwrap_throw().forget()
    }

    ///
    #[inline]
    pub fn target(&self) -> &EventTarget {
        &self.target
    }

    ///
    #[inline]
    pub fn event_type(&self) -> &'a str {
        &self.event_type
    }

    ///
    #[inline]
    pub fn callback(&self) -> &Closure<FnMut(Event)> {
        self.callback.as_ref().unwrap_throw()
    }
}

impl<'a> Drop for EventListener<'a> {
    #[inline]
    fn drop(&mut self) {
        self.target.remove_event_listener_with_callback(self.event_type, self.callback.as_ref().unwrap_throw().as_ref().unchecked_ref()).unwrap_throw();
    }
}

impl<'a> std::fmt::Debug for EventListener<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EventListener")
            .field("target", &self.target)
            .field("event_type", &self.event_type)
            .field("callback", &"Closure { ... }")
            .finish()
    }
}
