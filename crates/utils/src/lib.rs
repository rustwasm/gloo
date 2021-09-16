mod errors;
use wasm_bindgen::UnwrapThrowExt;

/// Convenience function to avoid repeating expect logic.
pub fn window() -> web_sys::Window {
    web_sys::window().expect_throw("Can't find the global Window")
}

/// Convenience function to access the web_sys DOM document.
pub fn document() -> web_sys::Document {
    window().document().expect_throw("Can't find document")
}

/// Convenience function to access the web_sys history.
pub fn history() -> web_sys::History {
    window().history().expect_throw("Can't find history")
}
