---
sidebar_position: 1
title: Introduction
slug: /console
---

The JavaScript's `console` object provides access to the browser's console.
Using the `console` object in Rust/WASM directly is cumbersome as it requires JavaScript glue code.
This crate exists to solve this problem by providing a set of ergonomic Rust APIs to deal
with the browser console.

# Example

The following example logs text to the console using `console.log`

```no_run, rust
# use wasm_bindgen::JsValue;
use gloo_console::log;

let object = JsValue::from("any JsValue can be logged");
log!("text", object)
```
