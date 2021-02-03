# Reqwasm

![GitHub branch checks state](https://img.shields.io/github/checks-status/hamza1311/reqwasm/master)
[![crates.io](https://img.shields.io/crates/v/reqwasm.svg?style=flat)](https://crates.io/crates/reqwasm)
[![docs.rs](https://img.shields.io/docsrs/reqwasm)](https://docs.rs/reqwasm/)
![licence](https://img.shields.io/crates/l/reqwasm)

HTTP requests library for WASM Apps. It provides idiomatic Rust bindings for the `web_sys` `fetch` API

## Example

```rust
let resp = Request::get("/path")
    .send()
    .await
    .unwrap();
assert_eq!(resp.status(), 200);
```

## Contributions

Your PRs and Issues are welcome. Note that all the contribution submitted by you, shall be licensed as MIT or APACHE 2.0 at your choice, without any additional terms or conditions.
