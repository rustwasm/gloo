use example_markdown::MarkdownWorker;
use wasm_bindgen::prelude::*;

use gloo::utils::document;
use gloo::worker::Spawnable;

use wasm_bindgen_futures::spawn_local;

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

    let mut bridge =
        MarkdownWorker::spawner().spawn_with_loader("/example_markdown_worker_loader.js");

    spawn_local(async move {
        let content = bridge.run(MARKDOWN_CONTENT.to_owned()).await;
        root.set_inner_html(&content);
    });
}
