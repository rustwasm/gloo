#![cfg(not(target_arch = "wasm32"))]

use warp::Filter;

#[tokio::main]
async fn main() {
    let dir = std::env::args().nth(1).expect("expected a target dir.");

    let route = warp::fs::dir(dir).with(
        // We need a server that serves the request with cross origin resource sharing.
        warp::cors()
            .allow_method("GET")
            .allow_method("HEAD")
            .allow_method("OPTIONS")
            .allow_any_origin(),
    );

    warp::serve(route).run(([127, 0, 0, 1], 9999)).await;
}
