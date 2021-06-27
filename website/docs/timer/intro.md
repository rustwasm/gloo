---
sidebar_position: 1
title: Introduction
slug: /timer
---


Working with timers on the Web: `setTimeout` and `setInterval`.

These APIs come in two flavors:

1. a callback style (that more directly mimics the JavaScript APIs), and
2. a `Future`s and `Stream`s API.

### Timeouts

Timeouts fire once after a period of time (measured in milliseconds).

#### Timeouts with a Callback Function

```rust
use gloo_timers::callback::Timeout;

let timeout = Timeout::new(1_000, move || {
    // Do something after the one second timeout is up!
});

// Since we don't plan on cancelling the timeout, call `forget`.
timeout.forget();
```

#### Timeouts as `Future`s

With the `futures` feature enabled, a `future` module containing futures-based
timers is exposed.

