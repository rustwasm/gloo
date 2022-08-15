#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen;
extern crate wasm_bindgen_test;

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

use gloo_utils::json::{from_serde, into_serde};
use serde_derive::{Deserialize, Serialize};

#[wasm_bindgen(start)]
pub fn start() {
    panic!();
}

#[wasm_bindgen(module = "/tests/serde.js")]
extern "C" {
    fn verify_serde(val: JsValue) -> JsValue;
}

#[cfg(feature = "serde-serialize")]
#[wasm_bindgen_test]
fn it_works() {
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

    let js = from_serde("foo").unwrap();
    assert_eq!(js.as_string(), Some("foo".to_string()));

    let ret = verify_serde(
        from_serde(&SerdeFoo {
            a: 0,
            b: "foo".to_string(),
            c: None,
            d: SerdeBar { a: 1 },
        })
        .unwrap(),
    );
    let foo = into_serde::<SerdeFoo>(&ret).unwrap();
    assert_eq!(foo.a, 2);
    assert_eq!(foo.b, "bar");
    assert!(foo.c.is_some());
    assert_eq!(foo.c.as_ref().unwrap().a, 3);
    assert_eq!(foo.d.a, 4);
    assert_eq!(into_serde::<String>(&JsValue::from("bar")).unwrap(), "bar");
    assert_eq!(into_serde::<i32>(&JsValue::undefined()).ok(), None);
    assert_eq!(into_serde::<i32>(&JsValue::null()).ok(), None);
}
