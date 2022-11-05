use std::time::Duration;

use futures::{FutureExt, StreamExt};
use gloo::timers::future::sleep;
use gloo::worker::reactor::ReactorScope;

use futures::sink::SinkExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlSignal {
    Start,
    Stop,
}

#[gloo::worker::reactor]
pub async fn Prime(mut scope: ReactorScope<ControlSignal, u64>) {
    while let Some(m) = scope.next().await {
        if m == ControlSignal::Start {
            'inner: for i in 1.. {
                // This is not the most efficient way to calculate prime,
                // but this example is here to demonstrate how primes can be
                // calculated in ascending order.
                if primes::is_prime(i) {
                    scope.feed(i).await.unwrap();
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
