#[gloo::worker::oneshot]
fn Worker(input: u32) -> u32 {
    input
}

fn main() {}
