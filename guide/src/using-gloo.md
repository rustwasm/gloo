# Using Gloo

Gloo is a *modular* toolkit: Each of its crates can either be used via the
umbrella `gloo` crate, which re-exports all of them for a stream-lined, holistic
experience, or each crate can be used independently.

## Using the Whole Toolkit

Using the whole toolkit via the umbrella `gloo` crate lets you hit the ground
running, with everything at your fingertips. This is a good choice for people
making Web applications, or top-level crates that are compiled into Wasm
binaries.

### `Cargo.toml`

Add a `gloo` dependency to your `Cargo.toml`:

```toml
[dependencies]
gloo = "0.1"
```

### `src/lib.rs`

Use various bits of `gloo` via its submodules, for example timers and intervals
are in `gloo::timers` and event listeners are in `gloo::events`:

```rust
use gloo::{timers, events};

// ...
```

## Using a Single Crate

Each crate in the Gloo toolkit can also be used independently from the rest of
the toolkit. This is a good choice for people making library crates that are
intended to be used by other people making Web applications or top-level crates
that are compiled into Wasm binaries.

### `Cargo.toml`

If we want to use only the Gloo functionality that wraps `setTimeout`, we can
add `gloo-timers` to our dependencies in `Cargo.toml`:

```toml
[dependencies]
gloo-timers = "0.1"
```

### `src/lib.rs`

Next, import the functionality you need from the `gloo_timers` crate, and go to
town with it:

```rust
use gloo_timers::callback::Timeout;

// ...
```
