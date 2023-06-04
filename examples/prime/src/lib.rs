use std::time::Duration;

use futures::{FutureExt, StreamExt};
use gloo::timers::future::sleep;
use gloo::worker::reactor::{reactor, ReactorScope};

use futures::sink::SinkExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlSignal {
    Start,
    Stop,
}

#[reactor]
pub async fn Prime(mut scope: ReactorScope<ControlSignal, u64>) {
    while let Some(m) = scope.next().await {
        if m == ControlSignal::Start {
            'inner: for i in 1.. {
                // This is not the most efficient way to calculate prime,
                // but this example is here to demonstrate how primes can be
                // sent to the application in an ascending order.
                if primes::is_prime(i) {
                    scope.send(i).await.unwrap();
                }

                futures::select! {
                    m = scope.next() => {
                        if m == Some(ControlSignal::Stop) {
                            break 'inner;
                        }
                    },
                    _ = sleep(Duration::from_millis(100)).fuse() => {},
                }
            }
        }
    }
}
// wasm-bindgen-test does not support serving additional files
// and trunk serve does not support CORS.
//
// To run tests against web workers, a test server with CORS support needs to be set up
// with the following commands:
//
// trunk build examples/prime/index.html
// cargo run -p example-prime --bin example_prime_test_server -- examples/prime/dist
//
// wasm-pack test --headless --firefox examples/prime
#[cfg(test)]
mod tests {
    use super::*;

    use gloo::worker::Spawnable;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn prime_worker_works() {
        gloo::console::log!("running test");
        let mut bridge =
            Prime::spawner().spawn("http://127.0.0.1:9999/example_prime_worker.js");

        bridge.send(ControlSignal::Start).await.expect("failed to send start signal");

        sleep(Duration::from_millis(1050)).await;

        bridge.send(ControlSignal::Stop).await.expect("failed to send stop signal");

        // 5 primes should be sent in 1 second.
        let primes: Vec<_> = bridge.take(5).collect().await;
        assert_eq!(primes, vec![2, 3, 5, 7, 11]);
    }
}
