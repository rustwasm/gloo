<div align="center">

  <h1><code>gloo-storage</code></h1>

  <p>
    <a href="https://crates.io/crates/gloo-storage"><img src="https://img.shields.io/crates/v/gloo-storage.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/gloo-storage"><img src="https://img.shields.io/crates/d/gloo-storage.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/gloo-storage"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
  </p>

  <h3>
    <a href="https://docs.rs/gloo-storage">API Docs</a>
    <span> | </span>
    <a href="https://github.com/rustwasm/gloo/blob/master/CONTRIBUTING.md">Contributing</a>
    <span> | </span>
    <a href="https://discordapp.com/channels/442252698964721669/443151097398296587">Chat</a>
  </h3>

  <sub>Built with 🦀🕸 by <a href="https://rustwasm.github.io/">The Rust and WebAssembly Working Group</a></sub>
</div>

This crate provides wrappers for the
[Web Storage API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API)

The data is stored in JSON form. We use [`serde`](https://serde.rs) for
serialization and deserialization.
