use std::time::Duration;

use gloo_timers::future::sleep;
use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

use gloo_history::{HashHistory, History};
use gloo_utils::window;

wasm_bindgen_test_configure!(run_in_browser);

#[test]
async fn history_works() {
    let history = HashHistory::new();
    assert_eq!(history.location().path(), "/");
    assert_eq!(window().location().pathname().unwrap(), "/");
    assert_eq!(window().location().hash().unwrap(), "#/");

    history.push("/path-a");
    assert_eq!(history.location().path(), "/path-a");
    assert_eq!(window().location().pathname().unwrap(), "/");
    assert_eq!(window().location().hash().unwrap(), "#/path-a");

    history.replace("/path-b");
    assert_eq!(history.location().path(), "/path-b");
    assert_eq!(window().location().pathname().unwrap(), "/");
    assert_eq!(window().location().hash().unwrap(), "#/path-b");

    history.back();
    sleep(Duration::from_millis(100)).await;
    assert_eq!(history.location().path(), "/");
    assert_eq!(window().location().pathname().unwrap(), "/");
    assert_eq!(window().location().hash().unwrap(), "#/");

    history.forward();
    sleep(Duration::from_millis(100)).await;
    assert_eq!(history.location().path(), "/path-b");
    assert_eq!(window().location().pathname().unwrap(), "/");
    assert_eq!(window().location().hash().unwrap(), "#/path-b");
}
