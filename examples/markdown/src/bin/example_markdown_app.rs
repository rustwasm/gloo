use example_markdown::MarkdownWorker;
use wasm_bindgen::prelude::*;

use gloo::utils::document;
use gloo::worker::Spawnable;

use js_sys::Promise;
use wasm_bindgen_futures::{spawn_local, JsFuture};

static MARKDOWN_CONTENT: &str = r#"
## Hello

This content is *rendered* by a **web worker**.

"#;

fn main() {
    console_error_panic_hook::set_once();

    let root = document()
        .query_selector("#root")
        .ok()
        .flatten()
        .expect_throw("failed to query root element");

    let bridge = MarkdownWorker::spawner()
        .callback(move |m| {
            root.set_inner_html(&m);
        })
        .spawn_with_loader("/example_markdown_worker_loader.js");

    bridge.send(MARKDOWN_CONTENT.to_owned());

    spawn_local(async move {
        bridge.send(MARKDOWN_CONTENT.to_owned());

        // We need to hold the bridge until the worker resolves.
        let promise = Promise::new(&mut |_, _| {});
        let _ = JsFuture::from(promise).await;
    });
}
