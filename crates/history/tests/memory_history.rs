use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

use gloo_history::{History, MemoryHistory};

wasm_bindgen_test_configure!(run_in_browser);

#[test]
fn history_works() {
    let history = MemoryHistory::new();
    assert_eq!(history.location().path(), "/");

    history.push("/path-a");
    assert_eq!(history.location().path(), "/path-a");

    history.replace("/path-b");
    assert_eq!(history.location().path(), "/path-b");

    history.back();
    assert_eq!(history.location().path(), "/");

    history.forward();
    assert_eq!(history.location().path(), "/path-b");
}
