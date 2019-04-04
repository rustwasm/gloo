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
- [Workflow](#workflow)
  - [Proposing a Design](#proposing-a-design)
    - [Design Checklist](#design-checklist)
  - [Implementation and Feedback Cycle](#implementation-and-feedback-cycle)
    - [Implementation Checklist](#implementation-checklist)
- [Team Members](#team-members)

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

## Workflow

Designing APIs for Gloo, its utility crates, and interfaces between them takes a
lot of care. The design space is large, and there is a lot of prior art to
consider. When coming to consensus on a design, we use a simplified, informal
version of [our RFC process][rfcs], where we have design discussions inside the
Gloo issues tracker.

> Note: when fixing a bug in a semver-compatible way that doesn't add any new
> API surface (i.e. changes are purely internal) we can skip the design proposal
> part of this workflow, and jump straight to a pull request.

The graph below gives an overview of the workflow for proposing, designing,
implementing, and merging new crates and APIs into Gloo. Notably, we expect a
large amount of design discussion to happen up front in the issue thread for the
design proposal.

[![Graph showing the workflow of proposing, designing, and merging new crates and
APIs into Gloo](./new-design-workflow.png)](./new-design-workflow.png)

[rfcs]: https://github.com/rustwasm/rfcs

### Proposing a Design

Before writing pull requests, we should have a clear idea of what is required
for implementation. This means there should be a skeleton of the API in the form
of types and function/method signatures. We should have a clear idea of the
layers of APIs we are exposing, and how they are built upon one another.

Note that exploratory implementations outside of Gloo are encouraged during this
period to get a sense for the API's usage, but you shouldn't send a pull request
until the design has been accepted.

Before the design is accepted, at least two team members must have stated that
they are in favor of accepting the design in the issue thread.

[Here is an issue template you can use for proposing
designs.](https://github.com/rustwasm/gloo/issues/new?assignees=&labels=&template=propose_design.md&title=)

#### Design Checklist

Here is a checklist of some general design principles that Gloo crates and APIs
should follow:

* [ ] Crate's public interface follows the [Rust API Guidelines][api-guidelines].

* [ ] Callback-taking APIs are generic over `F: Fn(A) -> B` (or `FnMut` or
  `FnOnce`) instead of taking `wasm_bindgen::Closure`s or
  `js_sys::Function`s directly.

* [ ] If the API can be implemented as a Future / Stream, then it should first
  be implemented as a callback, with the callback API put into the `callback`
  submodule.

  Then the Future / Stream should be implemented using the callback API, and
  should be put into the `future` or `stream` submodule.

  Make sure that the callback and Future / Stream APIs properly support
  cancellation (if it is possible to do so).

* [ ] Uses nice Rust-y types and interfaces instead of passing around untyped
  `JsValue`s.

* [ ] Has `fn as_raw(&self) -> &web_sys::Whatever` functions to get the
  underlying raw `web_sys`, `js_sys`, or `JsValue` type. This provides an escape
  hatch for dropping down to raw `web_sys` bindings when an API isn't fully
  supported by the crate yet.

  Similar for `from_raw` constructors and `into_raw` conversion methods when
  applicable.

* [ ] There is a loose hierarchy with "mid-level" APIs (which are essentially
  thin wrappers over the low-level APIs), and "high-level" APIs (which make more
  substantial changes).

  As a general rule, the high-level APIs should be built on top of the mid-level
  APIs, which in turn should be built on top of the low-level APIs
  (e.g. `web_sys`)

  There are exceptions to this, but they have to be carefully decided on a
  case-by-case basis.

### Implementation and Feedback Cycle

Once we've accepted a design, we can move forward with implementation and
creating pull requests.

The implementation should generally be unsurprising, since we should have
already worked out most of the kinks during the earlier design discussions. If
there are significant new issues or concerns uncovered during implementation,
then these should be brought up in the design proposal discussion thread again,
and the evolved design reaffirmed with two team members signing off once
again.

If there are no new concerns uncovered, then the implementation just needs to be
checked over by at least one team member. They provide code review and feedback
on the implementation, then the feedback is addressed and pull request updated.
Once the pull request is in a good place and CI is passing, a team member may
approve the pull request and merge it into Gloo. If any team member raises
concerns with the implementation, they must be resolved before the pull request
is merged.

#### Implementation Checklist

Here is a checklist that all crate and API implementations in Gloo should
fulfill:

* [ ] The crate should be named `gloo-foobar`, located at `gloo/crates/foobar`,
  and re-exported from the umbrella Gloo crate like:

  ```rust
  // gloo/src/lib.rs

  pub use gloo_foobar as foobar;
  ```

* [ ] The `authors` entry in `gloo/crates/foobar/Cargo.toml` is "The Rust and
  WebAssembly Working Group".

* [ ] Uses [`unwrap_throw` and `expect_throw`][unwrap-throw] instead of normal
  `unwrap` and `expect`.

* [ ] Headless browser and/or Node.js tests via `wasm-pack test`.

* [ ] Uses `#![deny(missing_docs, missing_debug_implementations)]`.

* [ ] Crate's root module documentation has at least one realistic example.

* [ ] Crate has at least a brief description of how to use it in the Gloo guide
  (the `mdbook` located at `gloo/guide`).

[unwrap-throw]: https://docs.rs/wasm-bindgen/0.2.37/wasm_bindgen/trait.UnwrapThrowExt.html
[api-guidelines]: https://rust-lang-nursery.github.io/api-guidelines/

## Team Members

Team members sign off on design proposals and review pull requests to Gloo.

* `@fitzgen`
* `@Pauan`
* `@rylev`
* `@yoshuawuyts`
* `@David-OConnor`

If you make a handful of significant contributions to Gloo and participate in
design proposals, then maybe you should be a team member! Think you or someone
else would be a good fit? Reach out to us!
