use gloo_storage::{SessionStorage, Storage};
use serde::Deserialize;
use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[test]
fn get() {
    let key = "key";
    let value = "value";
    SessionStorage::set(key, value).unwrap();

    let obtained_value: String = SessionStorage::get(key).unwrap();

    assert_eq!(value, obtained_value)
}

#[derive(Deserialize)]
struct Data {
    key1: String,
    key2: String,
}

#[test]
fn get_all() {
    SessionStorage::set("key1", "value").unwrap();
    SessionStorage::set("key2", "value").unwrap();

    let data: Data = SessionStorage::get_all().unwrap();
    assert_eq!(data.key1, "value");
    assert_eq!(data.key2, "value");
}

#[test]
fn set_and_length() {
    SessionStorage::clear();
    assert_eq!(SessionStorage::length(), 0);
    SessionStorage::set("key", "value").unwrap();
    assert_eq!(SessionStorage::length(), 1);
    SessionStorage::clear();
    assert_eq!(SessionStorage::length(), 0);
}
