---
slug: release-0.3.0
title: Releasing v0.3.0
author: Muhammad Hamza
author_title: Maintainer of Gloo
author_url: https://github.com/hamza1311
author_image_url: https://avatars.githubusercontent.com/u/47357913?v=4
tags: [release]
---

The Gloo team is happy to announce a new, long overdue, version of Gloo: v0.3.0.
Gloo is a modular toolkit for building fast, reliable Web applications and libraries with Rust and WASM.

## What's new

This release focuses on adding new features and crates.

### New crates

#### `gloo-console`
 
`gloo-console` provides an ergonomic way to access the browser's console API using macros:

```rust
log!("text");
```

The formatting is done on the browser side. Any `JsValue` can be provided and it'll be logged as-is:

```rust
let object = JsValue::from("any JsValue can be logged");
log!(object);
```

Multiple values can also be provided:

```rust
let object = JsValue::from("Some JsValue");
log!("literal", object);
```

#### `gloo-dialogs`

`gloo-dialogs` provides wrappers for the following functions:

- [`alert`](https://developer.mozilla.org/en-US/docs/Web/API/Window/alert)
- [`confirm`](https://developer.mozilla.org/en-US/docs/Web/API/Window/confirm)
- [`prompt`](https://developer.mozilla.org/en-US/docs/Web/API/Window/prompt)

```rust
alert("Hello World!");
```

```rust
prompt("What do you want to say?");
```


```rust
confirm("Are you sure?");
```

#### `gloo-render`

`gloo-render` provides wrapper for 
[`requestAnimationFrame`](https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame):

```rust
request_animation_frame(|_| {
    // inside animation frame.
})
```

#### `gloo-storage`

`gloo-storage` provides wrappers for the [Web Storage API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API).
It can be used to access both local storage and session storage.

### Other changes

- We now use GitHub Actions instead of Azure for CI
- READMEs and crate level docs are no longer synced
- This website exists!!

## Looking for contributors

Gloo project is in need of contributors. I recently became maintainer of this project, and I'm trying to revive it.
It would be really appreciated if you could contribute or raise awareness about the Gloo project.
