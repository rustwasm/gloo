use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

#[cfg(feature = "query")]
mod utils;

#[cfg(feature = "query")]
mod feat_serialize {
    use super::*;

    use utils::delayed_assert_eq;

    use wasm_bindgen_test::wasm_bindgen_test as test;

    use std::rc::Rc;

    use serde::{Deserialize, Serialize};

    use gloo_history::{BrowserHistory, History};

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
    async fn history_serialize_works() {
        let history = BrowserHistory::new();

        {
            let history = history.clone();
            delayed_assert_eq(|| history.location().path().to_owned(), || "/").await;
        }

        history.push("/path-a");

        {
            let history = history.clone();
            delayed_assert_eq(|| history.location().path().to_owned(), || "/path-a").await;
        }

        history.replace("/path-b");

        {
            let history = history.clone();
            delayed_assert_eq(|| history.location().path().to_owned(), || "/path-b").await;
        }

        history.back();

        {
            let history = history.clone();
            delayed_assert_eq(|| history.location().path().to_owned(), || "/").await;
        }

        history.forward();

        {
            let history = history.clone();
            delayed_assert_eq(|| history.location().path().to_owned(), || "/path-b").await;
        }

        history
            .push_with_query(
                "/path",
                Query {
                    a: "something".to_string(),
                    b: 123,
                },
            )
            .unwrap();

        {
            let history = history.clone();
            delayed_assert_eq(|| history.location().path().to_owned(), || "/path").await;
        }

        {
            let history = history.clone();
            delayed_assert_eq(
                || history.location().query_str().to_owned(),
                || "?a=something&b=123",
            )
            .await;
        }

        {
            let history = history.clone();
            delayed_assert_eq(
                || history.location().query::<Query>().unwrap(),
                || Query {
                    a: "something".to_string(),
                    b: 123,
                },
            )
            .await;
        }

        history.push_with_state(
            "/path-c",
            State {
                i: "something".to_string(),
                ii: 123,
            },
        );

        {
            let history = history.clone();
            delayed_assert_eq(|| history.location().path().to_owned(), || "/path-c").await;
        }

        {
            let history = history.clone();
            delayed_assert_eq(
                || history.location().state::<State>().unwrap(),
                || {
                    Rc::new(State {
                        i: "something".to_string(),
                        ii: 123,
                    })
                },
            )
            .await;
        }
    }
}
