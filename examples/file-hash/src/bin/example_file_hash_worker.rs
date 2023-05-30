use example_file_hash::codec::TransferrableCodec;
use example_file_hash::HashWorker;

use gloo_worker::Registrable;

fn main() {
    console_error_panic_hook::set_once();
    HashWorker::registrar()
        .encoding::<TransferrableCodec>()
        .register();
}
