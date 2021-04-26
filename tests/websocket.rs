use futures::{SinkExt, StreamExt};
use reqwasm::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

const ECHO_SERVER_URL: &str = env!("ECHO_SERVER_URL");

#[wasm_bindgen_test]
async fn websocket_works() {
    let ws = reqwasm::WebSocket::open(ECHO_SERVER_URL).unwrap();

    let (mut sender, mut receiver) = (ws.sender, ws.receiver);

    sender
        .send(Message::Text("test".to_string()))
        .await
        .unwrap();

    // ignore the first message
    let _ = receiver.next().await;
    assert_eq!(
        receiver.next().await.unwrap().unwrap(),
        Message::Text("test".to_string())
    )
}
