use std::sync::Arc;

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use log::{error, info, warn};
use tokio::sync::broadcast;
use warp::Filter;

use rotascope_core::shared::protocol::{ClientMessage, SwitchDirection};

pub fn ws_route(
    tx: Arc<broadcast::Sender<Bytes>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let tx_filter = warp::any().map(move || tx.clone());
    warp::path("ws")
        .and(warp::ws())
        .and(tx_filter)
        .map(|ws: warp::ws::Ws, tx: Arc<broadcast::Sender<Bytes>>| {
            ws.on_upgrade(move |socket| client_connection(socket, tx))
        })
}

pub async fn client_connection(ws: warp::ws::WebSocket, tx: Arc<broadcast::Sender<Bytes>>) {
    info!("websocket client connected");
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = tx.subscribe();

    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(bytes) => {
                    if let Err(e) = ws_tx.send(warp::ws::Message::binary(bytes)).await {
                        error!("websocket send error: {e:?}");
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    warn!("client lagged; skipped {count} frame(s)");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) if msg.is_text() => {
                    handle_client_command(msg.to_str().unwrap_or(""));
                }
                Ok(msg) if msg.is_binary() => {
                    handle_client_binary(msg.as_bytes());
                }
                Ok(msg) if msg.is_close() => break,
                Ok(_) => {}
                Err(e) => {
                    error!("websocket receive error: {e:?}");
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    info!("websocket client disconnected");
}

fn handle_client_command(text: &str) {
    match serde_json::from_str::<ClientMessage>(text) {
        Ok(message) => {
            match message {
                ClientMessage::SensorData { rotation_x, rotation_y, rotation_z } => {
                    info!("sensor data: x={rotation_x:.2}, y={rotation_y:.2}, z={rotation_z:.2}");
                }
                ClientMessage::SwitchDisplay { direction } => {
                    let dir_str = match direction {
                        SwitchDirection::Next => "next",
                        SwitchDirection::Previous => "previous",
                    };
                    info!("switch display: {dir_str}");
                }
                ClientMessage::TouchEvent(touch) => {
                    info!("touch event: {touch:?}");
                }
                ClientMessage::Heartbeat => {
                    // Silent
                }
            }
        }
        Err(e) => {
            warn!("failed to parse client message: {e}");
        }
    }
}

fn handle_client_binary(_data: &[u8]) {
    // Binary data from client (future use)
}
