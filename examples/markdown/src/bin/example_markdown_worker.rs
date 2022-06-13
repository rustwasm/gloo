use example_markdown::MarkdownWorker;

use gloo::worker::Registrable;

fn main() {
    console_error_panic_hook::set_once();

    MarkdownWorker::register();
}
