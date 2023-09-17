#![cfg(not(target_arch = "wasm32"))]

use warp::reply::with_header;
use warp::Filter;

// This server is purely to faclitate testing.
// Please read the instruction in lib.rs about how to run tests.
//
// If you are not running tests, you can simply ignore this file.
#[tokio::main]
async fn main() {
    let dir = std::env::args().nth(1).expect("expected a target dir.");

    let route = warp::fs::dir(dir)
        .with(
            // We need a server that serves the request with cross origin resource sharing.
            warp::cors()
                .allow_method("GET")
                .allow_method("HEAD")
                .allow_method("OPTIONS")
                .allow_any_origin(),
        )
        .map(|m| with_header(m, "cross-origin-resource-policy", "cross-origin"));

    println!("Test server is running at: http://127.0.0.1:9999/");

    warp::serve(route).run(([127, 0, 0, 1], 9999)).await;
}
