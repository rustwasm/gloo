// #![deny(missing_docs, missing_debug_implementations)]

mod timer;
pub mod externs;
pub mod macros;
mod counter;

pub use counter::Counter;
pub use timer::Timer;

#[doc(hidden)]
pub mod __macro {
    pub use wasm_bindgen::JsValue;
    pub use js_sys::Array;
    use wasm_bindgen::UnwrapThrowExt;
    use std::iter::FromIterator;

    pub fn table_with_data_and_columns<'a>(data: impl serde::Serialize, columns: impl IntoIterator<Item = &'a str>) {
        let data = JsValue::from_serde(&data).unwrap_throw();
        let columns = Array::from_iter(columns.into_iter().map(|it| JsValue::from_str(it)));

        crate::externs::table_with_data_and_columns(data, columns);
    }
}
