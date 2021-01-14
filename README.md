# Reqwasm

HTTP requests library for WASM Apps

## Example

```rust
let resp = Request::get("/path")
    .send()
    .await
    .unwrap();
assert_eq!(resp.status(), 200);
```
