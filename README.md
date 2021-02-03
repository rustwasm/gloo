# Reqwasm

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
