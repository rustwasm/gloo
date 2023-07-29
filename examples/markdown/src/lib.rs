use gloo::worker::oneshot::oneshot;
use pulldown_cmark::{html, Parser};

#[oneshot]
pub async fn MarkdownWorker(input: String) -> String {
    let parser = Parser::new(&input);

    let mut output = String::new();
    html::push_html(&mut output, parser);

    output
}

// wasm-bindgen-test does not support serving additional files
// and trunk serve does not support CORS.
//
// To run tests against web workers, a test server with CORS support needs to be set up
// with the following commands:
//
// trunk build examples/markdown/index.html
// cargo run -p example-markdown --bin example_markdown_test_server -- examples/markdown/dist
//
// wasm-pack test --headless --firefox examples/markdown
#[cfg(test)]
mod tests {
    use super::*;

    use gloo::worker::Spawnable;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    static MARKDOWN_CONTENT: &str = r#"
## Hello

This content is *rendered* by a **web worker**.

"#;

    #[wasm_bindgen_test]
    async fn markdown_worker_works() {
        let mut bridge =
            MarkdownWorker::spawner().spawn("http://127.0.0.1:9999/example_markdown_worker.js");

        let content = bridge.run(MARKDOWN_CONTENT.to_owned()).await;

        assert_eq!(
            &content,
            r#"<h2>Hello</h2>
<p>This content is <em>rendered</em> by a <strong>web worker</strong>.</p>
"#
        );
    }
}
