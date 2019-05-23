<div align="center">

  <h1><code>gloo-console-timer</code></h1>

  <p>
    <a href="https://travis-ci.org/rustasync/gloo-console-timer"><img src="https://img.shields.io/azure-devops/build/rustwasm/gloo/6.svg?style=flat-square" alt="Build Status" /></a>
    <a href="https://crates.io/crates/gloo-console-timer"><img src="https://img.shields.io/crates/v/gloo-console-timer.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/gloo-console-timer"><img src="https://img.shields.io/crates/d/gloo-console-timer.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/gloo-console-timer"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
  </p>

  <h3>
    <a href="https://docs.rs/gloo-console-timer">API Docs</a>
    <span> | </span>
    <a href="https://github.com/rustwasm/gloo/blob/master/CONTRIBUTING.md">Contributing</a>
    <span> | </span>
    <a href="https://discordapp.com/channels/442252698964721669/443151097398296587">Chat</a>
  </h3>

  <sub>Built with 🦀🕸 by <a href="https://rustwasm.github.io/">The Rust and WebAssembly Working Group</a></sub>
</div>


The `console.time` and `console.timeEnd` functions allow you to log the
timing of named operations to the browser's developer tools console. You
call `console.time("foo")` when the operation begins, and call
`console.timeEnd("foo")` when it finishes.

Additionally, these measurements will show up in your browser's profiler's
"timeline" or "waterfall" view.

[See MDN for more info](https://developer.mozilla.org/en-US/docs/Web/API/console#Timers).

This API wraps both the `time` and `timeEnd` calls into a single type
named `ConsoleTimer`, ensuring both are called.

### Scoped Measurement

Wrap code to be measured in a closure with `ConsoleTimer::scope`.

```rust
use gloo_console_timer::ConsoleTimer;

let value = ConsoleTimer::scope("foo", || {
    // Place code to be measured here
    // Optionally return a value.
});
```

### RAII-Style Measurement

For scenarios where `ConsoleTimer::scope` can't be used, like with
asynchronous operations, you can use `ConsoleTimer::new` to create a timer.
The measurement ends when the timer object goes out of scope / is dropped.

```rust
use gloo_console_timer::ConsoleTimer;
use gloo_timers::callback::Timeout;

// Start timing a new operation.
let timer = ConsoleTimer::new("foo");

// And then asynchronously finish timing.
let timeout = Timeout::new(1_000, move || {
    drop(timer);
});
```
