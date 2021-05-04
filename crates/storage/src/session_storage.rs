use wasm_bindgen::UnwrapThrowExt;

use crate::Storage;

/// Provides API to deal with `sessionStorage`
#[derive(Debug)]
pub struct SessionStorage;

impl Storage for SessionStorage {
    fn raw() -> web_sys::Storage {
        web_sys::window()
            .expect_throw("no window")
            .session_storage()
            .expect_throw("failed to get session_storage")
            .expect_throw("no session storage")
    }
}
