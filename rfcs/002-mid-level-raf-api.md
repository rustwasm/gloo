# Mid-Level requestAnimationFrame Api

## Summary

This is a mid-level API wrapper around the requestAnimationFrame JavaScript API. The possibility for a higher level API is left open.

## The API

The public API is a `Raf` struct, created with `FnMut(f64)` as a parameter to `Raf::new()`. 

This provided callback is immediately scheduled to be executed on each frame tick. The native requestAnimationFrame call returns an id upon scheduling, and `Raf` keeps this most-recent id in local state. Cancellation is handled via a `Drop` trait: further callbacks are prevented from being scheduled, and the last scheduled callback is cancelled via the native cancelAnimationFrame API.

Proposed code is as follows:

```rust
struct Raf {
    state: Rc<RefCell<Option<RafState>>>,
}

struct RafState {
    id: i32,
    closure: Closure<dyn FnMut(f64)>,
}

impl Raf {
    fn new<F>(mut callback: F) -> Self where F: FnMut(f64) + 'static {
        let state: Rc<RefCell<Option<RafState>>> = Rc::new(RefCell::new(None));

        fn schedule(callback: &Closure<dyn FnMut(f64)>) -> i32 {
            window()
                .unwrap_throw()
                .request_animation_frame(callback.as_ref().unchecked_ref())
                .unwrap_throw()
        }

        let closure = {
            let state = state.clone();

            Closure::wrap(Box::new(move |time| {
                {
                    let mut state = state.borrow_mut();
                    let state = state.as_mut().unwrap_throw();
                    state.id = schedule(&state.closure);
                }

                callback(time);
            }) as Box<dyn FnMut(f64)>)
        };

        *state.borrow_mut() = Some(RafState {
            id: schedule(&closure),
            closure
        });

        Self { state }
    }
}

impl Drop for Raf {
    fn drop(&mut self) {
        // The take is necessary in order to prevent an Rc leak
        let state = self.state.borrow_mut().take().unwrap_throw();

        window()
            .unwrap_throw()
            .cancel_animation_frame(state.id)
            .unwrap_throw();
    }
}

```

## Prior Art

The initial commit of this RFC uses the code verbatim from [dominator](https://github.com/Pauan/rust-dominator/) (with permission from the author)

The wasm-bindgen guide proposes [another approach](https://rustwasm.github.io/wasm-bindgen/examples/request-animation-frame.html) which is similar in spirit. However, adapting this into a generic API that supports cancellation would likely require that the user call a `cancel()` function explicitly, rather than the RAII style proposed here.