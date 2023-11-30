#![cfg(feature = "query")]

#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};
#[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))]
wasm_bindgen_test_configure!(run_in_browser);

use gloo_history::query::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct SimpleQuery {
    string: String,
    number: u64,
    optional: Option<String>,
    boolean: bool,
}

#[test]
fn test_raw_encode_simple() {
    let query = Raw("name=value&other=that");
    assert_eq!(query.to_query().unwrap(), "name=value&other=that");
}

#[test]
fn test_raw_decode_simple() {
    let query = "name=value&other=that";
    let decoded = <Raw<String>>::from_query(query).unwrap();
    assert_eq!(decoded, query);
}

#[test]
fn test_urlencoded_encode_simple() {
    let query = SimpleQuery {
        string: "test".into(),
        number: 42,
        optional: None,
        boolean: true,
    };

    let encoded = query.to_query().unwrap();
    assert_eq!(encoded, "string=test&number=42&boolean=true");
}

#[test]
fn test_urlencoded_decode_simple() {
    let encoded = "string=test&number=42&boolean=true";
    let data = SimpleQuery::from_query(encoded).unwrap();
    assert_eq!(
        data,
        SimpleQuery {
            string: "test".into(),
            number: 42,
            optional: None,
            boolean: true,
        }
    );
}
