use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU32, Ordering};

use wasm_bindgen::throw_str;

pub(crate) fn get_id() -> u32 {
    static ID_CTR: AtomicU32 = AtomicU32::new(0);

    ID_CTR.fetch_add(1, Ordering::SeqCst)
}

pub(crate) fn assert_absolute_path(path: &str) {
    if !path.starts_with('/') {
        throw_str("You cannot use relative path with this history type.");
    }
}

pub(crate) fn assert_no_query(path: &str) {
    if path.contains('?') {
        throw_str("You cannot have query in path, try use a variant of this method with `_query`.");
    }
}

pub(crate) fn assert_no_fragment(path: &str) {
    if path.contains('#') {
        throw_str("You cannot use fragments (hash) in memory history.");
    }
}

pub(crate) type WeakCallback = Weak<dyn Fn()>;

pub(crate) fn notify_callbacks(callbacks: Rc<RefCell<Vec<WeakCallback>>>) {
    let callables = {
        let mut callbacks_ref = callbacks.borrow_mut();

        // Any gone weak references are removed when called.
        let (callbacks, callbacks_weak) = callbacks_ref.iter().cloned().fold(
            (Vec::new(), Vec::new()),
            |(mut callbacks, mut callbacks_weak), m| {
                if let Some(m_strong) = m.clone().upgrade() {
                    callbacks.push(m_strong);
                    callbacks_weak.push(m);
                }

                (callbacks, callbacks_weak)
            },
        );

        *callbacks_ref = callbacks_weak;

        callbacks
    };

    for callback in callables {
        callback()
    }
}
