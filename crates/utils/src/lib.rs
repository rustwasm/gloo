pub mod errors;
use wasm_bindgen::UnwrapThrowExt;

/// Convenience function to avoid repeating expect logic.
pub fn window() -> web_sys::Window {
    web_sys::window().expect_throw("Can't find the global Window")
}

/// Convenience function to access the web_sys DOM document.
pub fn document() -> web_sys::Document {
    window().document().expect_throw("Can't find document")
}

/// Convenience function to access `document.body`.
pub fn body() -> web_sys::HtmlElement {
    document().body().expect_throw("Can't find document body")
}

/// Convenience function to access `document.documentElement`.
pub fn document_element() -> web_sys::Element {
    document()
        .document_element()
        .expect_throw("Can't find document element")
}

/// Convenience function to access the web_sys history.
pub fn history() -> web_sys::History {
    window().history().expect_throw("Can't find history")
}
