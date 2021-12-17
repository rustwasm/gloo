use std::time::Duration;

use gloo_timers::future::sleep;
use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

use gloo_history::{BrowserHistory, History};

wasm_bindgen_test_configure!(run_in_browser);

#[test]
async fn history_works() {
    let history = BrowserHistory::new();
    assert_eq!(history.location().path(), "/");

    history.push("/path-a");
    assert_eq!(history.location().path(), "/path-a");

    history.replace("/path-b");
    assert_eq!(history.location().path(), "/path-b");

    history.back();
    sleep(Duration::from_millis(100)).await;
    assert_eq!(history.location().path(), "/");

    history.forward();
    sleep(Duration::from_millis(100)).await;
    assert_eq!(history.location().path(), "/path-b");
}
