//! Crate that provides wrapper for
//! [requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)

#![deny(missing_docs, missing_debug_implementations)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

/// Handle for [`request_animation_frame`].
#[derive(Debug)]
pub struct AnimationFrame {
    render_id: i32,
    closure: Closure<dyn Fn(JsValue)>,
    is_done: Rc<RefCell<bool>>
}

impl Drop for AnimationFrame {
    fn drop(&mut self) {
        if !(*self.is_done.borrow()) {
            web_sys::window()
                .unwrap_throw()
                .cancel_animation_frame(self.render_id)
                .unwrap_throw()
        }
    }
}

/// Calls browser's `requestAnimationFrame`. It is cancelled when the handler is dropped.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)
pub fn request_animation_frame<F>(callback: F) -> AnimationFrame
where
    F: Fn(f64) + 'static,
{
    let is_done = Rc::new(RefCell::new(false));

    let callback = {
        let is_done = Rc::clone(&is_done);
        move |v: JsValue| {
            let time: f64 = v.as_f64().unwrap_or(0.0);
            callback(time);
            *is_done.borrow_mut() = true
        }
    };

    let callback: Closure<dyn Fn(JsValue)> =
        Closure::wrap(Box::new(callback) as Box<dyn Fn(JsValue)>);
    let render_id = web_sys::window()
        .unwrap_throw()
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .unwrap_throw();

    AnimationFrame {
        render_id,
        closure: callback,
        is_done,
    }
}
