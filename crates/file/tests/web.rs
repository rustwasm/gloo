//! Test suite for the Web and headless browsers.

use futures_rs::channel::mpsc;
use futures_rs::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

use gloo_file::{callbacks::read_as_text, Blob, File, ObjectUrl};
use web_sys::{window, Response};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn blob_from_str() {
    let (sender, mut receiver) = mpsc::unbounded();

    let blob = Blob::new("hello");

    let _reader = read_as_text(&blob, move |res| {
        sender.unbounded_send(res).unwrap();
    });

    assert_eq!(receiver.next().await.unwrap().unwrap(), "hello");
}

#[wasm_bindgen_test]
async fn file_from_str() {
    let (sender, mut receiver) = mpsc::unbounded();

    let file = File::new("a file", "hello");
    assert_eq!(file.name(), "a file");

    let _reader = read_as_text(&file, move |res| {
        sender.unbounded_send(res).unwrap();
    });

    assert_eq!(receiver.next().await.unwrap().unwrap(), "hello");
}

// TODO add ArrayBuffer test. This is hard at the moment because setting values of a TypedArray is
// not implemented in js-sys. [A PR to add them](https://github.com/rustwasm/wasm-bindgen/pull/2001)

#[wasm_bindgen_test]
fn png_file() {
    let file = File::new_with_options("img.png", PNG_FILE, Some("image/png"), None);
    assert_eq!(file.name(), "img.png");
    assert_eq!(file.raw_mime_type(), "image/png");
}

#[wasm_bindgen_test]
fn modified_since() {
    use chrono::Utc;
    use std::time::{Duration, SystemTime};

    let file = File::new("test.txt", "");
    let now: SystemTime = Utc::now().into();
    // Check the file was created less than 1ms ago. This should be plenty of time.
    assert!(
        file.last_modified_time()
            .checked_sub(Duration::from_millis(1))
            .unwrap()
            < now.into()
    );
}

#[cfg(feature = "futures")]
#[wasm_bindgen_test]
async fn text_future() {
    let blob = Blob::new("hello");

    assert_eq!(
        gloo_file::futures::read_as_text(&blob).await.unwrap(),
        "hello"
    );
}

#[cfg(feature = "futures")]
#[wasm_bindgen_test]
async fn data_url_future() {
    let blob = Blob::new_with_options(PNG_FILE, Some("image/png"));
    assert_eq!(
        gloo_file::futures::read_as_data_url(&blob).await.unwrap(),
        PNG_FILE_DATA
    );
}

#[cfg(feature = "futures")]
#[wasm_bindgen_test]
async fn bytes_future() {
    let blob = Blob::new_with_options(PNG_FILE, Some("image/png"));
    assert_eq!(
        gloo_file::futures::read_as_bytes(&blob).await.unwrap(),
        PNG_FILE
    );
}

const PNG_FILE: &'static [u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x01, 0x03, 0x00, 0x00, 0x00, 0x25, 0xdb, 0x56,
    0xca, 0x00, 0x00, 0x00, 0x03, 0x50, 0x4c, 0x54, 0x45, 0xff, 0x4d, 0x00, 0x5c, 0x35, 0x38, 0x7f,
    0x00, 0x00, 0x00, 0x01, 0x74, 0x52, 0x4e, 0x53, 0xcc, 0xd2, 0x34, 0x56, 0xfd, 0x00, 0x00, 0x00,
    0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x62, 0x00, 0x00, 0x00, 0x06, 0x00, 0x03, 0x36,
    0x37, 0x7c, 0xa8, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

#[cfg(feature = "futures")]
const PNG_FILE_DATA: &'static str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAA\
     Al21bKAAAAA1BMVEX/TQBcNTh/AAAAAXRSTlPM0jRW/QAAAApJREFUeJxjYgAAAAYAAzY3fKgAAAAASUVORK5CYII=";

#[cfg(feature = "futures")]
#[wasm_bindgen_test]
async fn blob_to_url() {
    let blob = Blob::new("hello world");
    let object_url = ObjectUrl::from(blob);
    // simulate a fetch, and expect to get a string containing the content back
    let request: JsFuture = window().unwrap().fetch_with_str(&object_url).into();
    let response = request.await.unwrap().unchecked_into::<Response>();
    let body: JsFuture = response.blob().unwrap().into();
    let body = body.await.unwrap().unchecked_into::<web_sys::Blob>();

    let body = gloo_file::futures::read_as_text(&body.into())
        .await
        .unwrap();
    assert_eq!(&body, "hello world");
}
