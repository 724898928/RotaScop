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

pub async fn run(
    jpeg_tx: Arc<broadcast::Sender<Bytes>>,
    h264_tx: Option<Arc<broadcast::Sender<Bytes>>>,
) -> Result<()> {
    let addr = ([0, 0, 0, 0], DEFAULT_PORT);
    info!("WebSocket server listening on ws://0.0.0.0:{}/ws", DEFAULT_PORT);
    warp::serve(ws_route(jpeg_tx, h264_tx).with(warp::cors().allow_any_origin()))
        .run(addr)
        .await;
    Ok(())
}

pub fn ws_route(
    jpeg_tx: Arc<broadcast::Sender<Bytes>>,
    h264_tx: Option<Arc<broadcast::Sender<Bytes>>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let jpeg_filter = warp::any().map(move || jpeg_tx.clone());
    let h264_filter = warp::any().map(move || h264_tx.clone());
    warp::path("ws")
        .and(warp::ws())
        .and(jpeg_filter)
        .and(h264_filter)
        .and(warp::path::full())
        .map(
            |ws: warp::ws::Ws,
             jpeg: Arc<broadcast::Sender<Bytes>>,
             h264: Option<Arc<broadcast::Sender<Bytes>>>,
             path: warp::path::FullPath| {
                let codec = path
                    .as_str()
                    .split('?')
                    .nth(1)
                    .unwrap_or("")
                    .split('&')
                    .find_map(|p| p.strip_prefix("codec="))
                    .unwrap_or("jpeg")
                    .to_string();
                ws.on_upgrade(move |socket| client_connection(socket, jpeg, h264, codec))
            },
        )
}

async fn client_connection(
    ws: warp::ws::WebSocket,
    jpeg_tx: Arc<broadcast::Sender<Bytes>>,
    h264_tx: Option<Arc<broadcast::Sender<Bytes>>>,
    codec: String,
) {
    info!("WebSocket client connected (codec: {codec})");
    let (mut ws_tx, mut ws_rx) = ws.split();

    let use_h264 = codec == "h264" && h264_tx.is_some();
    let rx: broadcast::Receiver<Bytes> = if use_h264 {
        info!("Client requested H.264 stream");
        h264_tx.as_ref().unwrap().subscribe()
    } else {
        jpeg_tx.subscribe()
    };

    let config = serde_json::json!({
        "type": "DisplayConfig",
        "total_displays": 1,
        "current_display": 0,
        "resolutions": [[1920, 1080]],
        "codec": if use_h264 { "h264" } else { "jpeg" }
    });
    if let Err(e) = ws_tx.send(warp::ws::Message::text(config.to_string())).await {
        warn!("Failed to send DisplayConfig: {e:?}");
        return;
    }

    let send_task = tokio::spawn(async move {
        let mut rx = rx;
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
                        Err(broadcast::error::RecvError::Lagged(_count)) => {}
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
            ClientMessage::Heartbeat => {}
        },
        Err(_) => {
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
