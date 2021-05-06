//! Crate that provides wrapper for
//! [requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)

#![deny(missing_docs, missing_debug_implementations)]

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Handle for [`request_animation_frame`].
#[derive(Debug)]
pub struct AnimationFrame {
    render_id: i32,
    closure: Closure<dyn Fn(JsValue)>,
}

impl Drop for AnimationFrame {
    fn drop(&mut self) {
        web_sys::window()
            .unwrap_throw()
            .cancel_animation_frame(self.render_id)
            .unwrap_throw()
    }
}

/// Calls browser's `requestAnimationFrame`. It is cancelled when the handler is dropped.
///
/// [MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestAnimationFrame)
pub fn request_animation_frame<F>(callback: F) -> AnimationFrame
where
    F: Fn(f64) + 'static,
{
    let callback = move |v: JsValue| {
        let time: f64 = v.as_f64().unwrap_or(0.0);
        callback(time)
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
    }
}
