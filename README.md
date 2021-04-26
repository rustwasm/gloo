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
let ws = WebSocket::open("wss://echo.websocket.org").unwrap();

let (mut sender, mut receiver) = (ws.sender, ws.receiver);

spawn_local(async move {
    while let Some(m) = receiver.next().await {
        match m {
            Ok(Message::Text(m)) => console_log!("message", m),
            Ok(Message::Bytes(m)) => console_log!("message", format!("{:?}", m)),
            Err(e) => {}
        }
    }
});

spawn_local(async move {
    sender.send(Message::Text("test".to_string())).await.unwrap();
})
```
## Contributions

Your PRs and Issues are welcome. Note that all the contribution submitted by you, shall be licensed as MIT or APACHE 2.0 at your choice, without any additional terms or conditions.
