use gloo::worker::Registrable;
use prime::Prime;

fn main() {
    console_error_panic_hook::set_once();

    Prime::registrar().register();
}
