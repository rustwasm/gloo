use std::cell::RefCell;

use gloo_worker::Codec;
use js_sys::{Array, JsString, Reflect};
use wasm_bindgen::JsCast;
use web_sys::File;

use crate::HashInput;

thread_local! {
    static FILE_STORE: RefCell<Option<File>> = RefCell::new(None);
}

pub struct TransferrableCodec {}

impl TransferrableCodec {
    pub fn pre_encode_input(input: &HashInput) {
        let file = input.file.clone();
        FILE_STORE.with(|m| m.replace(file));
    }

    pub fn post_decode_input(input: &mut HashInput) {
        let f = FILE_STORE.with(|m| m.take());
        input.file = f;
    }
}

impl Codec for TransferrableCodec {
    fn encode<I>(input: I) -> wasm_bindgen::JsValue
    where
        I: serde::Serialize,
    {
        let i = serde_wasm_bindgen::to_value(&input).expect("failed to encode");
        // This relys on some internal implementation details about gloo worker message types.
        // This should be considered as a last resort approach after all other possibilities are exhausted.
        if i.is_object() {
            if let Ok(m) = Reflect::get(&i, &JsString::from("ProcessInput")) {
                if let Ok(m) = m.dyn_into::<Array>() {
                    // HandlerID is ignored here. If it is possible to have multiple handles,
                    // Please consider using a hash map keyed by handler id for file store.
                    let i = m.get(1);

                    if i.is_object() {
                        if let Some(f) = FILE_STORE.with(|m| m.take()) {
                            Reflect::set(&i, &JsString::from("file"), &f)
                                .expect("failed to store file.");
                        }
                    }
                }
            }
        }

        i
    }

    fn decode<O>(input: wasm_bindgen::JsValue) -> O
    where
        O: for<'de> serde::Deserialize<'de>,
    {
        if input.is_object() {
            if let Ok(m) = Reflect::get(&input, &JsString::from("ProcessInput")) {
                if let Ok(m) = m.dyn_into::<Array>() {
                    let i = m.get(1);

                    if i.is_object() {
                        let f: Option<web_sys::File> = Reflect::get(&i, &JsString::from("file"))
                            .expect("failed to read file.")
                            .dyn_into()
                            .ok();

                        FILE_STORE.with(move |m| m.replace(f));
                    }
                }
            }
        }

        serde_wasm_bindgen::from_value(input).expect("failed to decode")
    }
}
