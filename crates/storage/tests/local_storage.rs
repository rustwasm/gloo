use gloo_storage::{LocalStorage, Storage};
use serde::Deserialize;
use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[test]
fn get() {
    let key = "key";
    let value = "value";
    LocalStorage::set(key, value).unwrap();

    let obtained_value: String = LocalStorage::get(key).unwrap();

    assert_eq!(value, obtained_value)
}

#[derive(Deserialize)]
struct Data {
    key1: String,
    key2: String,
}

#[test]
fn get_all() {
    LocalStorage::set("key1", "value").unwrap();
    LocalStorage::set("key2", "value").unwrap();

    let data: Data = LocalStorage::get_all().unwrap();
    assert_eq!(data.key1, "value");
    assert_eq!(data.key2, "value");
}

#[test]
fn set_and_length() {
    LocalStorage::clear();
    assert_eq!(LocalStorage::length(), 0);
    LocalStorage::set("key", "value").unwrap();
    assert_eq!(LocalStorage::length(), 1);
    LocalStorage::clear();
    assert_eq!(LocalStorage::length(), 0);
}
