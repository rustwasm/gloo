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
