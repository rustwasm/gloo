use gloo_net::http::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const HTTPBIN_URL: &str = env!("HTTPBIN_URL");

#[wasm_bindgen_test]
async fn query_preserve_initial() {
    let resp = Request::get(&format!("{}/get?key=value", HTTPBIN_URL))
        .query([("q", "val")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.url(), format!("{}/get?key=value&q=val", HTTPBIN_URL));
}

#[wasm_bindgen_test]
async fn query_preserve_duplicate_params() {
    let resp = Request::get(&format!("{}/get", HTTPBIN_URL))
        .query([("q", "1"), ("q", "2")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.url(), format!("{}/get?q=1&q=2", HTTPBIN_URL));
}

#[wasm_bindgen_test]
fn query_params_entries() {
    let params = QueryParams::new();
    params.append("a", "1");
    params.append("b", "value");
    let mut entries = params.entries();
    assert_eq!(entries.next(), Some(("a".into(), "1".into())));
    assert_eq!(entries.next(), Some(("b".into(), "value".into())));
    assert_eq!(entries.next(), None);
}

#[wasm_bindgen_test]
fn query_params_get() {
    let params = QueryParams::new();
    params.append("a", "1");
    params.append("a", "value");
    assert_eq!(params.get("a"), Some("1".to_string()));
    assert!(params.get("b").is_none());
    assert_eq!(
        params.get_all("a"),
        vec!["1".to_string(), "value".to_string()]
    );
}

#[wasm_bindgen_test]
fn query_params_delete() {
    let params = QueryParams::new();
    params.append("a", "1");
    params.append("a", "value");
    params.delete("a");
    assert!(params.get("a").is_none());
}

#[wasm_bindgen_test]
fn query_params_escape() {
    let params = QueryParams::new();
    params.append("a", "1");
    assert_eq!(params.to_string(), "a=1".to_string());

    params.append("key", "ab&c");
    assert_eq!(params.to_string(), "a=1&key=ab%26c");
}
