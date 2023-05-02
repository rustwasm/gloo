use gloo_net::http::Request;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

static HTTPBIN_URL: Lazy<&'static str> =
    Lazy::new(|| option_env!("HTTPBIN_URL").expect("Did you set HTTPBIN_URL?"));

#[wasm_bindgen_test]
async fn fetch() {
    let resp = Request::get(&format!("{}/get", *HTTPBIN_URL))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[wasm_bindgen_test]
async fn fetch_json() {
    #[derive(Deserialize, Debug)]
    struct HttpBin {
        url: String,
    }

    let url = format!("{}/get", *HTTPBIN_URL);
    let resp = Request::get(&url).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let json: HttpBin = resp.json().await.unwrap();
    assert_eq!(json.url, url);
}

#[wasm_bindgen_test]
async fn auth_valid_bearer() {
    let resp = Request::get(&format!("{}/get", *HTTPBIN_URL))
        .header("Authorization", "Bearer token")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}

#[wasm_bindgen_test]
async fn gzip_response() {
    #[derive(Deserialize, Debug)]
    struct HttpBin {
        gzipped: bool,
    }

    let resp = Request::get(&format!("{}/gzip", *HTTPBIN_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let json: HttpBin = resp.json().await.unwrap();
    assert!(json.gzipped);
}

#[wasm_bindgen_test]
async fn json_body() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Payload {
        data: String,
        num: i16,
    }

    let result = Request::post(&format!("{}/anything", *HTTPBIN_URL)).json(&Payload {
        data: "data".to_string(),
        num: 42,
    });
    assert!(result.is_ok(), "failed to create json request body")
}

#[wasm_bindgen_test]
async fn post_json() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Payload {
        data: String,
        num: i16,
    }

    #[derive(Deserialize, Debug)]
    struct HttpBin {
        json: Payload,
    }

    let req = Request::post(&format!("{}/anything", *HTTPBIN_URL))
        .json(&Payload {
            data: "data".to_string(),
            num: 42,
        })
        .expect("should not fail to serialize json")
        .send()
        .await
        .unwrap();

    assert_eq!(req.status(), 200);
    let resp: HttpBin = req.json().await.unwrap();
    assert_eq!(resp.json.data, "data");
    assert_eq!(resp.json.num, 42);
}

#[wasm_bindgen_test]
async fn fetch_binary() {
    #[derive(Deserialize, Debug)]
    struct HttpBin {
        data: String,
    }

    let resp = Request::post(&format!("{}/post", *HTTPBIN_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let json = resp.binary().await.unwrap();
    let json: HttpBin = serde_json::from_slice(&json).unwrap();
    assert_eq!(json.data, ""); // default is empty string
}

#[wasm_bindgen_test]
async fn query_preserve_initial() {
    let resp = Request::get(&format!("{}/get?key=value", *HTTPBIN_URL))
        .query([("q", "val")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.url(), format!("{}/get?key=value&q=val", *HTTPBIN_URL));
}

#[wasm_bindgen_test]
async fn query_preserve_duplicate_params() {
    let resp = Request::get(&format!("{}/get", *HTTPBIN_URL))
        .query([("q", "1"), ("q", "2")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.url(), format!("{}/get?q=1&q=2", *HTTPBIN_URL));
}

#[wasm_bindgen_test]
async fn request_clone() {
    let req = Request::get(&format!("{}/get", *HTTPBIN_URL))
        .build()
        .unwrap();

    let req1 = req.clone();
    let req2 = req.clone();

    let resp1 = req1.send().await.unwrap();
    let resp2 = req2.send().await.unwrap();

    assert_eq!(resp1.status(), 200);
    assert_eq!(resp2.status(), 200);
}

#[wasm_bindgen_test]
async fn request_clone_with_body() {
    // Get a stream of bytes by making a request
    let resp = Request::get(&format!("{}/get", *HTTPBIN_URL))
        .send()
        .await
        .unwrap();

    // Build a request with the body of the response
    let req = Request::post(&format!("{}/post", *HTTPBIN_URL))
        .body(resp.body())
        .build()
        .unwrap();

    // Clone the request
    let req1 = req.clone();
    let req2 = req.clone();

    // Send both requests
    let resp1 = req1.send().await.unwrap();
    let resp2 = req2.send().await.unwrap();

    assert_eq!(resp1.status(), 200);
    assert_eq!(resp2.status(), 200);
}
