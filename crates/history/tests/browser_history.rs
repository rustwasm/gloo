use wasm_bindgen_test::{wasm_bindgen_test as test, wasm_bindgen_test_configure};

use gloo_history::{BrowserHistory, History};

wasm_bindgen_test_configure!(run_in_browser);

mod utils;
use utils::delayed_assert_eq;

#[test]
async fn history_works() {
    let history = BrowserHistory::new();
    {
        let history = history.clone();
        delayed_assert_eq(move || history.location().path().to_owned(), || "/").await;
    }

    history.push("/path-a");

    {
        let history = history.clone();
        delayed_assert_eq(move || history.location().path().to_owned(), || "/path-a").await;
    }

    history.replace("/path-b");

    {
        let history = history.clone();
        delayed_assert_eq(move || history.location().path().to_owned(), || "/path-b").await;
    }

    history.back();

    {
        let history = history.clone();
        delayed_assert_eq(move || history.location().path().to_owned(), || "/").await;
    }

    history.forward();

    {
        let history = history.clone();
        delayed_assert_eq(move || history.location().path().to_owned(), || "/path-b").await;
    }
}
