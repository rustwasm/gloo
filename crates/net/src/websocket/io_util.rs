use core::cmp;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::io;

use futures_core::{ready, Stream as _};
use futures_io::{AsyncRead, AsyncWrite};
use futures_sink::Sink;

use crate::websocket::futures::WebSocket;
use crate::websocket::{Message as WebSocketMessage, WebSocketError};

impl WebSocket {
    /// Returns whether there are pending bytes left after calling [`AsyncRead::poll_read`] on this WebSocket.
    ///
    /// When calling [`AsyncRead::poll_read`], [`Stream::poll_next`](futures_core::Stream::poll_next) is called
    /// under the hood, and when the received item is too big to fit into the provided buffer, leftover bytes are
    /// stored. These leftover bytes are returned by subsequent calls to [`AsyncRead::poll_read`].
    #[cfg_attr(docsrs, doc(cfg(feature = "io-util")))]
    pub fn has_pending_bytes(&self) -> bool {
        self.read_pending_bytes.is_some()
    }
}

macro_rules! try_in_poll_io {
    ($expr:expr) => {{
        match $expr {
            Ok(o) => o,
            // WebSocket is closed, nothing more to read or write
            Err(WebSocketError::ConnectionClose(event)) if event.was_clean => {
                return Poll::Ready(Ok(0));
            }
            Err(e) => return Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
        }
    }};
}

#[cfg_attr(docsrs, doc(cfg(feature = "io-util")))]
impl AsyncRead for WebSocket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut data = if let Some(data) = self.as_mut().get_mut().read_pending_bytes.take() {
            data
        } else {
            match ready!(self.as_mut().poll_next(cx)) {
                Some(item) => match try_in_poll_io!(item) {
                    WebSocketMessage::Text(s) => s.into_bytes(),
                    WebSocketMessage::Bytes(data) => data,
                },
                None => return Poll::Ready(Ok(0)),
            }
        };

        let bytes_to_copy = cmp::min(buf.len(), data.len());
        buf[..bytes_to_copy].copy_from_slice(&data[..bytes_to_copy]);

        if data.len() > bytes_to_copy {
            data.drain(..bytes_to_copy);
            self.get_mut().read_pending_bytes = Some(data);
        }

        Poll::Ready(Ok(bytes_to_copy))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "io-util")))]
impl AsyncWrite for WebSocket {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // try flushing preemptively
        let _ = AsyncWrite::poll_flush(self.as_mut(), cx);

        // make sure sink is ready to send
        try_in_poll_io!(ready!(self.as_mut().poll_ready(cx)));

        // actually submit new item
        try_in_poll_io!(self.start_send(WebSocketMessage::Bytes(buf.to_vec())));
        // ^ if no error occurred, message is accepted and queued when calling `start_send`
        // (i.e.: `to_vec` is called only once)

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let res = ready!(Sink::poll_flush(self, cx));
        Poll::Ready(ws_result_to_io_result(res))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let res = ready!(Sink::poll_close(self, cx));
        Poll::Ready(ws_result_to_io_result(res))
    }
}

fn ws_result_to_io_result(res: Result<(), WebSocketError>) -> io::Result<()> {
    match res {
        Ok(()) => Ok(()),
        Err(WebSocketError::ConnectionClose(_)) => Ok(()),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn check_read_write() {
        let ws_echo_server_url =
            option_env!("WS_ECHO_SERVER_URL").expect("Did you set WS_ECHO_SERVER_URL?");

        let mut ws = WebSocket::open(ws_echo_server_url).unwrap();

        // ignore first message
        // the echo-server uses it to send it's info in the first message
        let _ = ws.next().await.unwrap();

        let (mut reader, mut writer) = AsyncReadExt::split(ws);

        spawn_local(async move {
            writer.write_all(b"test 1").await.unwrap();
            writer.write_all(b"test 2").await.unwrap();
        });

        spawn_local(async move {
            let mut buf = [0u8; 6];
            reader.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"test 1");
            reader.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"test 2");
        });
    }

    #[wasm_bindgen_test]
    async fn with_pending_bytes() {
        let ws_echo_server_url =
            option_env!("WS_ECHO_SERVER_URL").expect("Did you set WS_ECHO_SERVER_URL?");

        let mut ws = WebSocket::open(ws_echo_server_url).unwrap();

        // ignore first message
        // the echo-server uses it to send it's info in the first message
        let _ = ws.next().await.unwrap();

        ws.write_all(b"1234567890").await.unwrap();

        let mut buf = [0u8; 5];

        ws.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"12345");
        assert!(ws.has_pending_bytes());

        ws.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"67890");
        assert!(!ws.has_pending_bytes());
    }
}
