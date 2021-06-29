// #![deny(missing_docs, missing_debug_implementations)]

mod counter;
pub mod externs;
pub mod macros;
mod timer;

pub use counter::Counter;
pub use timer::Timer;

#[doc(hidden)]
pub mod __macro {
    pub use js_sys::Array;
    pub use wasm_bindgen::JsValue;
    use wasm_bindgen::UnwrapThrowExt;

    pub fn table_with_data_and_columns<'a>(
        data: impl serde::Serialize,
        columns: impl IntoIterator<Item = &'a str>,
    ) {
        let data = JsValue::from_serde(&data).unwrap_throw();
        let columns = columns
            .into_iter()
            .map(|it| JsValue::from_str(it))
            .collect();

        crate::externs::table_with_data_and_columns(data, columns);
    }
}
