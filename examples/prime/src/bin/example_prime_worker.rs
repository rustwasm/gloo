use example_prime::Prime;
use gloo::worker::Registrable;

fn main() {
    console_error_panic_hook::set_once();

    Prime::registrar().register();
}
