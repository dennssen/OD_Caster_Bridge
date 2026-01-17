use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::RwLock;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use crate::gui::app::AppState;

pub async fn run_websocket_server(state: Arc<RwLock<AppState>>) {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("WebSocket server listening on ws://{}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let state_clone = Arc::clone(&state);
        tokio::spawn(handle_client(stream, state_clone));
    }
}

async fn handle_client(stream: TcpStream, state: Arc<RwLock<AppState>>) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake error: {}", e);
            return;
        }
    };

    {
        let mut s = state.write().await;
        s.connected_clients += 1;
    }

    let (mut write, mut read) = ws_stream.split();

    // Handle incoming queries
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {}
            Ok(Message::Close(_)) => break,
            Ok(Message::Ping(ping)) => {
                let _ = write.send(Message::Pong(ping)).await;
            }
            Err(_) => break,
            _ => {}
        }
    }

    let mut s = state.write().await;
    if s.connected_clients > 0 {
        s.connected_clients -= 1;
    }
}