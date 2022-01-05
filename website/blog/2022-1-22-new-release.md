---
slug: release-0.6.0 
title: Releasing v0.6.0 
author: Muhammad Hamza 
author_title: Maintainer of Gloo
author_url: https://github.com/hamza1311
author_image_url: https://avatars.githubusercontent.com/u/47357913?v=4
tags: [release]
---

The Gloo team is happy to announce a new version of Gloo: v0.6.0. Gloo is a modular toolkit for building fast, reliable
Web applications and libraries with Rust and WASM.

## What's new

This release focuses on adding new features and crates.

### New crate: `gloo-worker`

Gloo workers are a way to offload tasks to web workers. These are run concurrently using
[web-workers](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers).

This feature has been requested and overdue for a while. Gloo-worker is made by 
moving [`yew-agent`](https://yew.rs/docs/concepts/agents) to Gloo, while making it framework independent in the process.
This allows us to have a neat abstraction over the browser's Web Workers API which can be consumed from anywhere.

### Features

This release has been light on new features. The only improvement is `gloo_utils` now providing a new wrapper 
to obtain the document `head`.

## Notable mention from last release

Last release, Gloo v0.5.0 did not receive its own blog post. That released introduced one major new crate: `gloo-history`
amongst other small improvements, which can be found in the [GitHub changelog](https://github.com/rustwasm/gloo/releases/tag/0.5.0).

### `gloo-history`

Gloo history provides wrappers for browser's history API. It exposes ergonomic Rust APIs for the browser's APIs which 
can be used to build other tools. In fact, [`yew-router`](https://github.com/yewstack/yew/pull/2239) has been 
reworked to use `gloo-history` under-the-hood.

## Looking for contributors

Gloo project is in need of contributors. It would be really appreciated if you could contribute or raise awareness about
the Gloo project.
