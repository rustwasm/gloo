use std::time::Duration;

use gloo_timers::future::sleep;
use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

use serde::{Deserialize, Serialize};

use gloo_history::{BrowserHistory, History, Location};

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Query {
    a: String,
    b: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct State {
    i: String,
    ii: u64,
}

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

    history
        .push_with_query(
            "/path",
            Query {
                a: "something".to_string(),
                b: 123,
            },
        )
        .unwrap();

    assert_eq!(history.location().path(), "/path");
    assert_eq!(history.location().search(), "?a=something&b=123");
    assert_eq!(
        history.location().query::<Query>().unwrap(),
        Query {
            a: "something".to_string(),
            b: 123,
        }
    );

    history
        .push_with_state(
            "/path-c",
            State {
                i: "something".to_string(),
                ii: 123,
            },
        )
        .unwrap();

    assert_eq!(history.location().path(), "/path-c");
    assert_eq!(
        history.location().state::<State>().unwrap(),
        State {
            i: "something".to_string(),
            ii: 123,
        }
    );
}
