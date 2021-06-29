//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use gloo_console::Timer;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn scoped_timer_returns_value() {
    let value = Timer::scope("foo", || true);

    assert!(value);
}
