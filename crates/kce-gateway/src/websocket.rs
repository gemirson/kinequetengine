//! WebSockets implementation for real-time telemetry (FT-031).

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

pub async fn start_websocket_server(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream) {
    let ws_stream = match accept_async(stream).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("WS handshake error: {}", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Echo for MVP
                let _ = ws_sender.send(Message::Text(format!("Echo: {}", text))).await;
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                tracing::error!("WS error: {}", e);
                break;
            }
            _ => (),
        }
    }
}
