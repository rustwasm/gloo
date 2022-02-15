# Reqwasm

[![crates.io](https://img.shields.io/crates/v/reqwasm.svg?style=flat)](https://crates.io/crates/reqwasm)
[![docs.rs](https://img.shields.io/docsrs/reqwasm)](https://docs.rs/reqwasm/)
![licence](https://img.shields.io/crates/l/reqwasm)

HTTP requests library for WASM Apps. It provides idiomatic Rust bindings for the `web_sys` `fetch` and `WebSocket` API

## Examples

### HTTP

```rust
let resp = Request::get("/path")
    .send()
    .await
    .unwrap();
assert_eq!(resp.status(), 200);
```

### WebSocket

```rust
use reqwasm::websocket::{Message, futures::WebSocket};
use wasm_bindgen_futures::spawn_local;
use futures::{SinkExt, StreamExt};

let mut ws = WebSocket::open("wss://echo.websocket.org").unwrap();
let (mut write, mut read) = ws.split();

spawn_local(async move {
    write.send(Message::Text(String::from("test"))).await.unwrap();
    write.send(Message::Text(String::from("test 2"))).await.unwrap();
});

spawn_local(async move {
    while let Some(msg) = read.next().await {
        console_log!(format!("1. {:?}", msg))
    }
    console_log!("WebSocket Closed")
})
```
## Contributions

Your PRs and Issues are welcome. Note that all the contribution submitted by you, shall be licensed as MIT or APACHE 2.0 at your choice, without any additional terms or conditions.
