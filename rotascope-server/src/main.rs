use std::env;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use image::{codecs::jpeg::JpegEncoder, RgbaImage};
use log::{error, info, warn};
use tokio::runtime::Runtime;
use tokio::sync::broadcast;
use tokio::time::sleep;
use warp::Filter;

const DEFAULT_PORT: u16 = 8083;
const TARGET_FPS: u64 = 15;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let (tx, _rx) = broadcast::channel::<Bytes>(16);
    let tx = Arc::new(tx);

    let capture_tx = tx.clone();
    thread::spawn(move || {
        let runtime = Runtime::new().expect("failed to create tokio runtime");
        runtime.block_on(async move {
            if let Err(error) = capture_loop(capture_tx).await {
                error!("capture loop stopped: {error:?}");
            }
        });
    });

    let tx_filter = warp::any().map(move || tx.clone());
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(tx_filter)
        .map(|ws: warp::ws::Ws, tx: Arc<broadcast::Sender<Bytes>>| {
            ws.on_upgrade(move |socket| client_connection(socket, tx))
        });

    let addr = ([0, 0, 0, 0], DEFAULT_PORT);
    info!("RotaScope server listening on ws://0.0.0.0:{DEFAULT_PORT}/ws");
    warp::serve(ws_route.with(warp::cors().allow_any_origin()))
        .run(addr)
        .await;

    Ok(())
}

async fn capture_loop(tx: Arc<broadcast::Sender<Bytes>>) -> Result<()> {
    let frame_duration = Duration::from_millis(1000 / TARGET_FPS);
    let monitor = select_monitor().context("failed to select monitor")?;

    loop {
        if tx.receiver_count() == 0 {
            sleep(Duration::from_millis(250)).await;
            continue;
        }

        let start = Instant::now();

        match monitor.capture_image() {
            Ok(frame) => {
                let width = frame.width();
                let height = frame.height();
                let Some(image) = RgbaImage::from_raw(width, height, frame.into_raw()) else {
                    error!("capture returned an invalid RGBA buffer");
                    continue;
                };

                let mut jpeg_bytes = Vec::with_capacity(200_000);
                let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, 72);

                if let Err(error) = encoder.encode_image(&image) {
                    error!("jpeg encode error: {error:?}");
                } else {
                    let _ = tx.send(Bytes::from(jpeg_bytes));
                }
            }
            Err(error) => {
                error!("capture error: {error:?}");
            }
        }

        let elapsed = start.elapsed();
        if elapsed < frame_duration {
            sleep(frame_duration - elapsed).await;
        }
    }
}

fn select_monitor() -> Result<xcap::Monitor> {
    let monitors = xcap::Monitor::all()?;
    if monitors.is_empty() {
        anyhow::bail!("no monitors found");
    }

    let selected_index = env::var("ROTASCOPE_DISPLAY_INDEX")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);

    let monitor_count = monitors.len();
    if selected_index >= monitor_count {
        warn!(
            "ROTASCOPE_DISPLAY_INDEX={} is out of range; using display 0 of {}",
            selected_index, monitor_count
        );
    }

    let index = selected_index.min(monitor_count - 1);
    let monitor = monitors
        .into_iter()
        .nth(index)
        .context("selected monitor disappeared")?;

    info!("Capturing display index {index}: {monitor:?}");
    Ok(monitor)
}

async fn client_connection(ws: warp::ws::WebSocket, tx: Arc<broadcast::Sender<Bytes>>) {
    info!("websocket client connected");
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = tx.subscribe();

    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(bytes) => {
                    if let Err(error) = ws_tx.send(warp::ws::Message::binary(bytes)).await {
                        error!("websocket send error: {error:?}");
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

    let receive_task = tokio::spawn(async move {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(message) if message.is_text() => {
                    info!("client command: {}", message.to_str().unwrap_or("<invalid utf8>"));
                }
                Ok(message) if message.is_close() => break,
                Ok(_) => {}
                Err(error) => {
                    error!("websocket receive error: {error:?}");
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }

    info!("websocket client disconnected");
}
