#![no_implicit_prelude]

#[::gloo::worker::reactor::reactor]
fn Worker(_scope: ::gloo::worker::reactor::ReactorScope<(), ()>) -> u32 {
    0
}

fn main() {}
