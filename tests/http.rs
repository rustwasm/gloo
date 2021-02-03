use reqwasm::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const HTTPBIN_URL: &str = env!("HTTPBIN_URL");

#[wasm_bindgen_test]
async fn fetch() {
    let resp = Request::get(&format!("{}/get", HTTPBIN_URL))
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

    let url = format!("{}/get", HTTPBIN_URL);
    let resp = Request::get(&url).send().await.unwrap();
    let json: HttpBin = resp.json().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(json.url, url);
}

#[wasm_bindgen_test]
async fn auth_valid_bearer() {
    let resp = Request::get(&format!("{}/get", HTTPBIN_URL))
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

    let resp = Request::get(&format!("{}/gzip", HTTPBIN_URL))
        .send()
        .await
        .unwrap();
    let json: HttpBin = resp.json().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(json.gzipped, true);
}

#[wasm_bindgen_test]
async fn post_json() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Payload {
        data: String,
    }

    #[derive(Deserialize, Debug)]
    struct HttpBin {
        json: Payload,
    }

    let resp = Request::post(&format!("{}/anything", HTTPBIN_URL))
        .body(
            serde_json::to_string(&Payload {
                data: "data".to_string(),
            })
            .unwrap(),
        )
        .send()
        .await
        .unwrap();
    let json: HttpBin = resp.json().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(json.json.data, "data");
}
