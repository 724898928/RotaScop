use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use rotascope_core::shared::protocol::{
    ClientMessage, SwitchDirection, TouchEvent,
};
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use warp::Filter;

use crate::input_injector;

const DEFAULT_PORT: u16 = 8083;

pub async fn run(capture_tx: Arc<broadcast::Sender<Bytes>>) -> Result<()> {
    let addr = ([0, 0, 0, 0], DEFAULT_PORT);
    info!("WebSocket server listening on ws://0.0.0.0:{}/ws", DEFAULT_PORT);
    warp::serve(ws_route(capture_tx).with(warp::cors().allow_any_origin()))
        .run(addr)
        .await;
    Ok(())
}

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

async fn client_connection(ws: warp::ws::WebSocket, tx: Arc<broadcast::Sender<Bytes>>) {
    info!("WebSocket client connected");
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = tx.subscribe();

    // Send initial DisplayConfig
    let config = serde_json::json!({
        "type": "DisplayConfig",
        "total_displays": 1,
        "current_display": 0,
        "resolutions": [[1920, 1080]]
    });
    if let Err(e) = ws_tx.send(warp::ws::Message::text(config.to_string())).await {
        warn!("Failed to send DisplayConfig: {e:?}");
        return;
    }

    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                result = rx.recv() => {
                    match result {
                        Ok(bytes) => {
                            if let Err(e) = ws_tx.send(warp::ws::Message::binary(bytes)).await {
                                error!("websocket send error: {e:?}");
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_count)) => {
                            // fast encoder outruns slow network; skip silently
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
            }
        }
    });

    let recv_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) if msg.is_text() => {
                    let text = msg.to_str().unwrap_or("").to_string();
                    handle_client_text(&text);
                }
                Ok(msg) if msg.is_close() => break,
                Ok(_) => {}
                Err(e) => {
                    warn!("websocket receive error: {e:?}");
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    info!("WebSocket client disconnected");
}

fn handle_client_text(text: &str) {
    match serde_json::from_str::<ClientMessage>(text) {
        Ok(message) => match message {
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
            ClientMessage::TouchEvent(event) => {
                handle_touch_event(&event);
            }
            ClientMessage::Heartbeat => {
                // Silent
            }
        },
        Err(_) => {
            // Might be a raw JSON that doesn't match enum, try parsing as generic
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(text) {
                match val["type"].as_str() {
                    Some("SensorData") | Some("SwitchDisplay") | Some("Heartbeat") => {}
                    Some("TouchEvent") => {
                        let event_type = val["event"].as_str().unwrap_or("move");
                        let x = val["x"].as_f64().unwrap_or(0.0) as f32;
                        let y = val["y"].as_f64().unwrap_or(0.0) as f32;
                        let event = match event_type {
                            "down" => TouchEvent::Down { x, y },
                            "up" => TouchEvent::Up { x, y },
                            "scroll" => TouchEvent::Scroll {
                                delta_x: val["delta_x"].as_f64().unwrap_or(0.0) as f32,
                                delta_y: val["delta_y"].as_f64().unwrap_or(0.0) as f32,
                            },
                            _ => TouchEvent::Move { x, y },
                        };
                        handle_touch_event(&event);
                    }
                    Some(other) => {
                        info!("unknown command type: {other}");
                    }
                    None => {}
                }
            }
        }
    }
}

fn handle_touch_event(event: &TouchEvent) {
    #[cfg(windows)]
    {
        if let Err(e) = input_injector::inject_input(event, 1920, 1080) {
            warn!("Failed to inject input: {e:?}");
        }
    }

    #[cfg(not(windows))]
    {
        let _ = event;
        info!("touch event received (input injection not supported on this platform)");
    }
}
