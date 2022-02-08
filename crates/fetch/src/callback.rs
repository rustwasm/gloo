use wasm_bindgen::prelude::*;
use wasm_bindgen_test::wasm_bindgen_test;

/// Create a HeaderMap out of a web `Headers` object.
fn convert_headers(headers: web_sys::Headers) -> http::HeaderMap {
    use http::{
        header::{HeaderName, HeaderValue},
        HeaderMap,
    };
    let mut map = HeaderMap::new();
    for (key, value) in headers.iter() {
        let key = HeaderName::from_bytes(key.as_bytes()).unwrap_throw();
        let value = HeaderValue::from_bytes(value.as_bytes()).unwrap_throw();
        map.insert(key, value);
    }
    map
}

#[wasm_bindgen_test]
fn test_convert_headers() {
    let headers = web_sys::Headers::new().unwrap_throw();
    headers.append("Content-Type", "text/plain").unwrap_throw();
    let header_map = convert_headers(headers);
    //panic!("{:?}", header_map);
}
