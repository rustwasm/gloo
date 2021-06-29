use wasm_bindgen::prelude::*;
use std::boxed::Box;
use js_sys::Array;

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

    // TODO console.group()
    // TODO console.groupCollapsed()
    // TODO console.groupEnd()

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
