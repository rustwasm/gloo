//! The JavaScript's `console` object provides access to the browser's console.
//! Using the `console` object in Rust/WASM directly is cumbersome as it requires JavaScript glue code.
//! This crate exists to solve this problem by providing a set of ergonomic Rust APIs to deal
//! with the browser console.
//!
//! # Example
//!
//! The following example logs text to the console using `console.log`
//!
//! ```no_run, rust
//! # use wasm_bindgen::JsValue;
//! use gloo_console::log;
//!
//! let object = JsValue::from("any JsValue can be logged");
//! log!("text", object)
//! ```

#![deny(missing_docs, missing_debug_implementations)]

mod console_dbg;
mod counter;
#[doc(hidden)]
pub mod externs;
mod macros;
mod timer;

pub use counter::Counter;
pub use macros::*;
pub use timer::Timer;

#[doc(hidden)]
pub mod __macro {
    use gloo_utils::format::JsValueSerdeExt;
    pub use js_sys::Array;
    pub use wasm_bindgen::JsValue;
    use wasm_bindgen::UnwrapThrowExt;

    pub fn table_with_data_and_columns<'a>(
        data: impl serde::Serialize,
        columns: impl IntoIterator<Item = &'a str>,
    ) {
        let data = <JsValue as JsValueSerdeExt>::from_serde(&data).unwrap_throw();
        let columns = columns.into_iter().map(JsValue::from_str).collect();

        crate::externs::table_with_data_and_columns(data, columns);
    }
}
