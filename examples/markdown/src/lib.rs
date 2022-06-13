use gloo::worker::{HandlerId, Worker, WorkerScope};

use pulldown_cmark::{html, Parser};

#[derive(Debug)]
pub enum Msg<T> {
    Respond { output: T, id: HandlerId },
}

pub struct MarkdownWorker {
    scope: WorkerScope<Self>,
}

impl Worker for MarkdownWorker {
    // The Markdown Markup to Render.
    type Input = String;

    type Message = Msg<String>;

    // The Rendered Html Output.
    type Output = String;

    fn create(scope: WorkerScope<Self>) -> Self {
        Self { scope }
    }

    fn update(&mut self, msg: Self::Message) {
        let Msg::Respond { output, id } = msg;

        self.scope.respond(id, output);
    }

    fn received(&mut self, msg: Self::Input, who: HandlerId) {
        let parser = Parser::new(&msg);

        let mut output = String::new();
        html::push_html(&mut output, parser);

        self.scope.send_message(Msg::Respond { output, id: who });
    }
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

    use js_sys::Promise;
    use wasm_bindgen_futures::{spawn_local, JsFuture};

    use futures::channel::oneshot;
    use std::cell::RefCell;

    wasm_bindgen_test_configure!(run_in_browser);

    static MARKDOWN_CONTENT: &str = r#"
## Hello

This content is *rendered* by a **web worker**.

"#;

    #[wasm_bindgen_test]
    async fn markdown_worker_works() {
        let (tx, rx) = oneshot::channel();

        let tx = RefCell::new(Some(tx));

        let bridge = MarkdownWorker::spawner()
            .callback(move |m| {
                if let Some(tx) = tx.borrow_mut().take() {
                    let _ = tx.send(m);
                }
            })
            .spawn("http://127.0.0.1:9999/example_markdown_worker.js");

        bridge.send(MARKDOWN_CONTENT.to_owned());

        spawn_local(async move {
            bridge.send(MARKDOWN_CONTENT.to_owned());

            // We need to hold the bridge until the worker resolves.
            let promise = Promise::new(&mut |_, _| {});
            let _ = JsFuture::from(promise).await;
        });

        let content = rx.await.unwrap();

        assert_eq!(
            &content,
            r#"<h2>Hello</h2>
<p>This content is <em>rendered</em> by a <strong>web worker</strong>.</p>
"#
        );
    }
}
