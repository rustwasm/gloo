# Contributing to Gloo

Hi! Thanks for your interest in contributing to Gloo — we'd love to have your
participation! If you want help or mentorship, reach out to us in a GitHub
issue, or on [the `#WG-wasm` channel of the Rust Discord][discord] and introduce
yourself.

[discord]: https://discord.gg/9e6Pvjz

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->


- [Code of Conduct](#code-of-conduct)
- [Building and Testing](#building-and-testing)
  - [Prerequisites](#prerequisites)
  - [Building](#building)
  - [Testing](#testing)
    - [Wasm Tests](#wasm-tests)
    - [Non-Wasm Tests](#non-wasm-tests)
  - [Formatting](#formatting)
- [Pull Requests](#pull-requests)
- [Gloo Crate Guidelines](#gloo-crate-guidelines)
- [Team](#team)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## Code of Conduct

We abide by the [Rust Code of Conduct][coc] and ask that you do as well.

[coc]: https://www.rust-lang.org/en-US/conduct.html

## Building and Testing

### Prerequisites

These tools are required for building and testing Gloo:

* [**The Rust toolchain:**][install-rust] `rustup`, `cargo`, `rustc`, etc.
* [**`rustfmt`:**][rustfmt] We use `rustfmt` for a consistent code style across
  the whole code base.
* [**`wasm-pack`:**][install-wasm-pack] We use `wasm-pack` to orchestrate
  headless browser testing.

[install-rust]: https://www.rust-lang.org/tools/install
[rustfmt]: https://github.com/rust-lang/rustfmt
[install-wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/

### Building

You can build every Gloo crate:

```
cargo build --all
```

Or you can build one particular crate:

```
cargo build -p my-particular-crate
```

### Testing

#### Wasm Tests

To run headless browser and/or Node.js tests for a particular crate:

```
wasm-pack test crates/my-particular-crate
```

#### Non-Wasm Tests

You can run the non-Wasm tests (e.g. doc tests) for every Gloo crate with:

```
cargo test --all
```

Or you can run one particular crate's non-Wasm tests:

```
cargo test -p my-particular-crate
```

### Formatting

To (re)format the Gloo source code, run:

```
$ cargo fmt --all
```

## Pull Requests

All pull requests must be reviewed and approved of by at least one [team](#team)
member before merging.

## Gloo Crate Guidelines

These are the guidelines for Gloo crates:

* [ ] Crate's public interface follows the [Rust API Guidelines][api-guidelines].

* [ ] Uses [`unwrap_throw` and `expect_throw`][unwrap-throw] instead of normal `unwrap` and
      `expect`.

* [ ] Callback-taking APIs are generic over `F: Fn(A) -> B` (or `FnMut` or
      `FnOnce`) instead of taking `wasm_bindgen::Closure`s or
      `js_sys::Function`s directly.

* [ ] Uses nice Rust-y types and interfaces instead of passing around untyped
      `JsValue`s.

* [ ] Has `fn as_raw(&self) -> &web_sys::Whatever` functions to get the
      underlying raw `web_sys`, `js_sys`, or `JsValue` type. This provides an
      escape hatch for dropping down to raw `web_sys` bindings when an API isn't
      fully supported by the crate yet.

* [ ] Headless browser and/or Node.js tests via `wasm-pack test`.

* [ ] Uses `#![deny(missing_docs, missing_debug_implementations)]`.

* [ ] Crate's root module documentation has at least one realistic example.

[unwrap-throw]: https://docs.rs/wasm-bindgen/0.2.37/wasm_bindgen/trait.UnwrapThrowExt.html
[api-guidelines]: https://rust-lang-nursery.github.io/api-guidelines/

## Team

| [<img alt="fitzgen" src="https://avatars2.githubusercontent.com/u/74571?s=117&v=4" width="117">](https://github.com/fitzgen) | | | |
|:---:|:---:|:---:|:---:|
| [`fitzgen`](https://github.com/fitzgen) | | | |
