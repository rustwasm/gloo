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
    - [Wasm Headless Browser Tests](#wasm-headless-browser-tests)
    - [Non-Wasm Tests](#non-wasm-tests)
  - [Formatting](#formatting)
- [Gloo Crate Checklist](#gloo-crate-checklist)
- [Designing APIs](#designing-apis)

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

#### Wasm Headless Browser Tests

To run headless browser tests for a particular crate:

```shell
wasm-pack test crates/my-particular-crate --headless --firefox # or --safari or --chrome
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

## Gloo Crate Checklist

Here is a checklist that all Gloo utility crates should fulfill:

* [ ] The crate should be named `gloo-foobar`, located at `crates/foobar`, and
      re-exported from the umbrella crate like:

      ```rust
      pub use gloo_foobar as foobar;
      ```

* [ ] Crate's public interface follows the [Rust API Guidelines][api-guidelines].

* [ ] Uses [`unwrap_throw` and `expect_throw`][unwrap-throw] instead of normal `unwrap` and
      `expect`.

* [ ] Callback-taking APIs are generic over `F: Fn(A) -> B` (or `FnMut` or
      `FnOnce`) instead of taking `wasm_bindgen::Closure`s or
      `js_sys::Function`s directly.

* [ ] If the API can be implemented as a Future / Stream, then it should first be implemented as a callback, with the callback API put into the `callback` submodule.

     Then the Future / Stream should be implemented using the callback API, and should be put into the `future` or `stream` submodule.

     Make sure that the callback and Future / Stream APIs properly support cancellation (if it is possible to do so).

* [ ] Uses nice Rust-y types and interfaces instead of passing around untyped
      `JsValue`s.

* [ ] Has `fn as_raw(&self) -> &web_sys::Whatever` functions to get the
      underlying raw `web_sys`, `js_sys`, or `JsValue` type. This provides an
      escape hatch for dropping down to raw `web_sys` bindings when an API isn't
      fully supported by the crate yet.

* [ ] There is a loose hierarchy with "mid-level" APIs (which are essentially thin wrappers over the low-level APIs), and "high-level" APIs (which make more substantial changes).

     As a general rule, the high-level APIs should be built on top of the mid-level APIs, which in turn should be built on top of the low-level APIs (e.g. `web_sys`)

* [ ] Headless browser and/or Node.js tests via `wasm-pack test`.

* [ ] Uses `#![deny(missing_docs, missing_debug_implementations)]`.

* [ ] Crate's root module documentation has at least one realistic example.

[unwrap-throw]: https://docs.rs/wasm-bindgen/0.2.37/wasm_bindgen/trait.UnwrapThrowExt.html
[api-guidelines]: https://rust-lang-nursery.github.io/api-guidelines/

## Designing APIs

Designing APIs for Gloo, its utility crates, and interfaces between them takes a
lot of care. The design space is large, and there is a lot of prior art to
consider. When coming to consensus on a design, we use a simplified, informal
version of [our RFC process][rfcs], where we have design discussions inside the
Gloo issues tracker.

[Here is an issue template you can use for proposing
designs.](https://github.com/rustwasm/gloo/issues/new?assignees=&labels=&template=propose_design.md&title=)

[rfcs]: https://github.com/rustwasm/rfcs
