use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

#[cfg(feature = "query")]
mod feat_serialize {
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use std::rc::Rc;

    use serde::{Deserialize, Serialize};

    use gloo_history::{History, MemoryHistory};

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
    fn history_serialize_works() {
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
        assert_eq!(history.location().query_str(), "?a=something&b=123");
        assert_eq!(
            history.location().query::<Query>().unwrap(),
            Query {
                a: "something".to_string(),
                b: 123,
            }
        );

        history.push_with_state(
            "/path-c",
            State {
                i: "something".to_string(),
                ii: 123,
            },
        );

        assert_eq!(history.location().path(), "/path-c");
        assert_eq!(
            history.location().state::<State>().unwrap(),
            Rc::new(State {
                i: "something".to_string(),
                ii: 123,
            })
        );
    }
}
