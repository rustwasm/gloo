---
slug: release-0.4.0 
title: Releasing v0.4.0 
author: Muhammad Hamza 
author_title: Maintainer of Gloo
author_url: https://github.com/hamza1311
author_image_url: https://avatars.githubusercontent.com/u/47357913?v=4
tags: [release]
---

The Gloo team is happy to announce a new version of Gloo: v0.4.0. Gloo is a modular toolkit for building fast, reliable
Web applications and libraries with Rust and WASM.

## What's new

This release focuses on adding new features and crates.

### Features

* `gloo-utils` crate: `gloo_utils` wraps common `web_sys` features to provide cleaner API for accessing `window`,
  working with JS Errors, etc.
* Add `dbg!` equivalent in `gloo_console` for easy console.log debugging.

### Fixes

* Remove the unnecessary copy in `Blob::new` ([#152](https://github.com/rustwasm/gloo/pull/152))
* Fix dir, dirxml macros in `gloo-console` ([#154](https://github.com/rustwasm/gloo/pull/154))

## Looking for contributors

Gloo project is in need of contributors. It would be really appreciated if you could contribute or raise awareness about
the Gloo project.
