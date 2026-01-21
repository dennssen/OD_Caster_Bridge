use std::collections::HashMap;
use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::RwLock;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
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
    let mut rx = {
        let s = state.read().await;
        s.broadcast_tx.subscribe()
    };

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

    {
        let s = state.read().await;
        if let Ok(json) = serde_json::to_string(&s.game_state) {
            let _ = write.send(Message::Text(Utf8Bytes::from(json))).await;
        }
    }

    loop {
        tokio::select! {
            Ok(game_state) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&game_state) {
                    if write.send(Message::Text(Utf8Bytes::from(json))).await.is_err() {
                        break;
                    }
                }
            }

            Some(msg) = read.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(query) = serde_json::from_str::<HashMap<String, String>>(&text) {
                            handle_query(&state, query).await;
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Ok(Message::Ping(ping)) => {
                        let _ = write.send(Message::Pong(ping)).await;
                    }
                    Err(_) => break,
                    _ => {}
                }
            }
        }
    }

    {
        let mut s = state.write().await;
        if s.connected_clients > 0 {
            s.connected_clients -= 1;
        }
    }
}

async fn handle_query(state: &Arc<RwLock<AppState>>, query: HashMap<String, String>) {
    if let Some(action) = query.get("action") {
        let mut s = state.write().await;

        match action.as_str() {
            "setSubscribedGamemode" => {
                if let Some(slot_id) = query.get("slotId") {
                    println!("Subscribing to gamemode: {}", slot_id);
                    s.subscribed_gamemode_slot_id = slot_id.clone();
                }
            }
            _ => {}
        }
    }
}