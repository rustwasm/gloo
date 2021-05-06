<div align="center">

  <h1><code>gloo-file</code></h1>

  <p>
    <a href="https://dev.azure.com/rustwasm/gloo/_build?definitionId=6"><img src="https://img.shields.io/azure-devops/build/rustwasm/gloo/6.svg?style=flat-square" alt="Build Status" /></a>
    <a href="https://crates.io/crates/gloo-file"><img src="https://img.shields.io/crates/v/gloo-file.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/gloo-file"><img src="https://img.shields.io/crates/d/gloo-file.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/gloo-file"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
  </p>

  <h3>
    <a href="https://docs.rs/gloo-file">API Docs</a>
    <span> | </span>
    <a href="https://github.com/rustwasm/gloo/blob/master/CONTRIBUTING.md">Contributing</a>
    <span> | </span>
    <a href="https://discordapp.com/channels/442252698964721669/443151097398296587">Chat</a>
  </h3>

  <sub>Built with 🦀🕸 by <a href="https://rustwasm.github.io/">The Rust and WebAssembly Working Group</a></sub>
</div>

Provides wrappers for working with `Blob`s and `File`s from JavaScript.

A `File` is just a `Blob` with some extra data: a name and a last modified time.

In the File API, `Blob`s are opaque objects that lazily fetch their contained data when
asked. This allows a `Blob` to represent some resource that isn't completely available, for
example a WebSocket message that is being received or a file that needs to be read from disk.

You can asynchronously access the contents of the `Blob` through callbacks,
but that is rather inconvenient, so this crate provides some functions which
return a `Future` instead.
