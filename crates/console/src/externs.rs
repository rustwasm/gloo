use js_sys::Array;
use std::boxed::Box;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn assert(assertion: bool, objs: Box<[JsValue]>);

    #[wasm_bindgen(js_namespace = console)]
    pub fn clear();

    // TODO console.count()
    // TODO console.countReset()

    #[wasm_bindgen(js_namespace = console, variadic)]
    pub fn debug(items: Box<[JsValue]>);

    #[wasm_bindgen(js_namespace = console)]
    pub fn dir(items: &JsValue);

    #[wasm_bindgen(js_namespace = console)]
    pub fn dirxml(items: &JsValue);

    #[wasm_bindgen(js_namespace = console, variadic)]
    pub fn error(items: Box<[JsValue]>);

    #[wasm_bindgen(js_namespace = console, variadic)]
    pub fn group(items: Box<[JsValue]>);

    #[wasm_bindgen(js_namespace = console, js_name = groupCollapsed, variadic)]
    pub fn group_collapsed(items: Box<[JsValue]>);

    #[wasm_bindgen(js_namespace = console, js_name = groupEnd)]
    pub fn group_end();

    #[wasm_bindgen(js_namespace = console, variadic)]
    pub fn info(items: Box<[JsValue]>);

    #[wasm_bindgen(js_namespace = console, variadic)]
    pub fn log(items: Box<[JsValue]>);

    #[wasm_bindgen(js_namespace = console, js_name = table)]
    pub fn table_with_data(data: JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = table)]
    pub fn table_with_data_and_columns(data: JsValue, columns: Array);

    #[wasm_bindgen(js_namespace = console, variadic)]
    pub fn trace(items: Box<[JsValue]>);

    #[wasm_bindgen(js_namespace = console, variadic)]
    pub fn warn(items: Box<[JsValue]>);

}
