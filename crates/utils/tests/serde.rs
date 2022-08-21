#![cfg(target_arch = "wasm32")]
#![cfg(feature = "serde")]
extern crate wasm_bindgen;
extern crate wasm_bindgen_test;

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use gloo_utils::format::JsValueSerdeExt;

use serde_derive::{Deserialize, Serialize};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(start)]
pub fn start() {
    panic!();
}

#[wasm_bindgen(module = "/tests/serde.js")]
extern "C" {
    fn verify_serde(val: JsValue);
    fn make_js_value() -> JsValue;
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SerdeFoo {
    a: u32,
    b: String,
    c: Option<SerdeBar>,
    d: SerdeBar,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SerdeBar {
    a: u32,
}

#[wasm_bindgen_test]
fn from_serde() {
    let js = JsValue::from_serde("foo").unwrap();
    assert_eq!(js.as_string(), Some("foo".to_string()));

    verify_serde(
        JsValue::from_serde(&SerdeFoo {
            a: 0,
            b: "foo".to_string(),
            c: None,
            d: SerdeBar { a: 1 },
        })
        .unwrap(),
    );
}

#[wasm_bindgen_test]
fn into_serde() {
    let js_value = make_js_value();
    let foo = js_value.into_serde::<SerdeFoo>().unwrap();
    assert_eq!(foo.a, 2);
    assert_eq!(foo.b, "bar");
    assert!(foo.c.is_some());
    assert_eq!(foo.c.as_ref().unwrap().a, 3);
    assert_eq!(foo.d.a, 4);

    assert_eq!(JsValue::from("bar").into_serde::<String>().unwrap(), "bar");
    assert_eq!(JsValue::undefined().into_serde::<i32>().ok(), None);
    assert_eq!(JsValue::null().into_serde::<i32>().ok(), None);
}
