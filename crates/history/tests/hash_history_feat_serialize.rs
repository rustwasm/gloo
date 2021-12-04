use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

#[cfg(all(feature = "query", feature = "state"))]
mod feat_serialize {
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use std::time::Duration;

    use serde::{Deserialize, Serialize};

    use gloo_history::{HashHistory, History, Location};

    use gloo_timers::future::sleep;
    use gloo_utils::window;

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
        let history = HashHistory::new();
        assert_eq!(history.location().path(), "/");

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
        assert_eq!(window().location().pathname().unwrap(), "/");
        assert_eq!(
            window().location().hash().unwrap(),
            "#/path?a=something&b=123"
        );
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
        assert_eq!(window().location().pathname().unwrap(), "/");
        assert_eq!(window().location().hash().unwrap(), "#/path-c");
        assert_eq!(
            history.location().state::<State>().unwrap(),
            State {
                i: "something".to_string(),
                ii: 123,
            }
        );
    }
}
