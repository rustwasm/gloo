use wasm_bindgen::UnwrapThrowExt;

use crate::Storage;

/// Provides API to deal with `localStorage`
#[derive(Debug)]
pub struct LocalStorage;

impl Storage for LocalStorage {
    fn raw() -> web_sys::Storage {
        web_sys::window()
            .expect_throw("no window")
            .local_storage()
            .expect_throw("failed to get local_storage")
            .expect_throw("no local storage")
    }
}
