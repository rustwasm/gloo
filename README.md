# Gloo

**A toolkit for building fast, reliable Web applications and libraries with Rust
and Wasm.**

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->


- [What?](#what)
- [Goals](#goals)
- [Example](#example)
- [Get Involved!](#get-involved)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

## What?

In [the Rust and WebAssembly working group's 2019 roadmap][roadmap], we chose to
deliberately cultivate our library ecosystem by building a modular toolkit:

> ## Collaborating on a Modular Toolkit
>
> > The idea of building [high-level libraries] in a modular way that will allow
> > others in the community to put the components together in a different way is
> > very exciting to me. This hopefully will make the ecosystem as a whole much
> > stronger.
> >
> > In particular I’d love to see a modular effort towards implementing a virtual
> > DOM library with JSX like syntax. There have been several efforts on this
> > front but all have seemed relatively monolithic and “batteries included”. I
> > hope this will change in 2019.
>
> <cite>&mdash; Ryan Levick in [Rust WebAssembly
> 2019](https://blog.ryanlevick.com/posts/rust-wasm-2019/)</cite>
>
> > Don't create branded silos. Branding might perhaps be useful to achieve
> > fame. But if we truly want Rust's Wasm story to succeed we should think of
> > ways to collaborate instead of carving out territory.
>
> <cite>&mdash; Yoshua Wuyts in [Wasm
> 2019](https://blog.yoshuawuyts.com/wasm-2019/)</cite>
>
> In 2018, we created foundational libraries like [`js-sys` and
> `web-sys`][announcing-web-sys]. In 2019, we should build modular, high-level
> libraries on top of them, and collect the libraries under an umbrella toolkit
> crate for a holistic experience. This toolkit and its libraries will make
> available all the batteries you want when targeting Wasm.
>
> Building a greenfield Web application? Use the whole toolkit to hit the ground
> running. Carefully crafting a tiny Wasm module and integrating it back into an
> existing JavaScript project? Grab that one targeted library you need out from
> the toolkit and use it by itself.

Gloo is this modular toolkit.

[announcing-web-sys]: https://rustwasm.github.io/2018/09/26/announcing-web-sys.html
[roadmap]: https://github.com/rustwasm/rfcs/pull/7

## Goals

* **Support both whole Web applications and small, targeted libraries:** Gloo,
  and the collection of utility crates that make up its toolkit, should help you
  be productive if you are writing a green-field web application with Rust and
  Wasm. And it should also help you be productive if you are writing a small,
  targeted Wasm library that will be integrated into an existing JavaScript
  application.

* **Cultivate the Rust and Wasm library ecosystem:** We want to use Gloo as a
  forcing function for creating and sharing the building blocks of Web
  development. The kinds of libraries that *any* framework or high-level library
  would need to build. We want to explicitly disentangle these libraries and
  make them available for sharing across the whole ecosystem.

* **Modular Toolkit, not Framework:** Gloo should be a loose collection of
  utility crates that can be used individually, or all together. Gloo doesn't
  assume that it "owns" the whole Webpage, that it controls the Wasm `start`
  function, etc. This lack of assumptions enables reaching more use cases (such
  as surgically replacing a hot code path from JS) than monolithic frameworks
  can. Wherever possible, Gloo should prefer interfaces over implementations, so
  that different implementations with different approaches are swap-able.

* **Fast:** Let's leverage Rust's zero-cost abstractions, and design with
  performance in mind, to show everyone how fast the Web can be ;)

* **Reliable:** Every crate should be thoroughly tested. Headless browser
  tests. Quickcheck tests. Using the type system to make whole classes of bugs
  impossible.

* **Small:** Small code size for faster page loads. No accidentally pulling in
  all of the panicking and formatting infrastructure. Users shouldn't have to
  make a trade off between using Gloo libraries and having small Wasm binaries.

* **Idiomatic:** We want to build Rust-y APIs, that feel natural to use. The
  Web's APIs were not designed for the Rust language, and you can feel the
  impedance mismatch every now and then. Let's correct that, bridge the gap, and
  make libraries that are a joy to use.

## Example

TODO: we haven't built enough of Gloo to have anything compelling here yet!

## Get Involved!

Want to help us build Gloo? Check out [`CONTRIBUTING.md`](./CONTRIBUTING.md)!
