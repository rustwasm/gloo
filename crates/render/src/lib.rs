//! Crate that provides wrapper for
//! [requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)

#![deny(missing_docs, missing_debug_implementations)]

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Handle for [`request_animation_frame`].
#[derive(Debug)]
pub struct AnimationFrame {
    render_id: i32,
    _closure: Closure<dyn Fn(JsValue)>,
    callback_wrapper: Rc<RefCell<Option<CallbackWrapper>>>,
}

struct CallbackWrapper(Box<dyn FnOnce(f64) + 'static>);
impl fmt::Debug for CallbackWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CallbackWrapper")
    }
}

impl Drop for AnimationFrame {
    fn drop(&mut self) {
        if self.callback_wrapper.borrow_mut().is_some() {
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
pub fn request_animation_frame<F>(callback_once: F) -> AnimationFrame
where
    F: FnOnce(f64) + 'static,
{
    let callback_wrapper = Rc::new(RefCell::new(Some(CallbackWrapper(Box::new(callback_once)))));
    let callback: Closure<dyn Fn(JsValue)> = {
        let callback_wrapper = Rc::clone(&callback_wrapper);
        Closure::wrap(Box::new(move |v: JsValue| {
            let time: f64 = v.as_f64().unwrap_or(0.0);
            let callback = callback_wrapper.borrow_mut().take().unwrap().0;
            callback(time);
        }))
    };

    let render_id = web_sys::window()
        .unwrap_throw()
        .request_animation_frame(callback.as_ref().unchecked_ref())
        .unwrap_throw();

    AnimationFrame {
        render_id,
        _closure: callback,
        callback_wrapper,
    }
}
