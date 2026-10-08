use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use crate::state::ReloadTx;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(tx): Extension<ReloadTx>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, tx))
}

async fn handle_socket(socket: WebSocket, tx: ReloadTx) {
    let mut rx = tx.subscribe();
    let (mut sender, mut receiver) = socket.split();

    // Forward broadcast messages to this WebSocket client
    let send_task = tokio::spawn(async move {
        loop {
            let msg = match rx.recv().await {
                Ok(msg) => msg,
                // A slow client missed some events: it cannot know which, so tell it to reload
                // everything and keep going. Ending the loop here would leave a socket that
                // looks open but never receives anything again.
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => r#"{"event":"reload"}"#.to_string(),
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            };
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Drain incoming frames (we don't use client→server messages)
    while receiver.next().await.is_some() {}
    send_task.abort();
}
