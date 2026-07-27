mod capture;
mod encoder;
mod input_injector;
mod quic_server;
mod utils;
mod virtual_display;
mod ws_server;

use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use tokio::signal;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0:8083")]
    ws_addr: String,

    #[arg(short = 'q', long)]
    quic_addr: Option<String>,

    #[arg(short = 'Q', long, default_value = "40")]
    quality: u8,

    #[arg(short = 'd', long, default_value = "0")]
    display: usize,

    #[arg(short = 'c', long, default_value = "jpeg")]
    codec: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let q = args.quality.clamp(10, 100);
    std::env::set_var("ROTASCOPE_QUALITY", q.to_string());
    std::env::set_var("ROTASCOPE_DISPLAY_INDEX", args.display.to_string());
    std::env::set_var("ROTASCOPE_CODEC", args.codec.clone());

    info!(
        "RotaScope v2 — target 60fps, JPEG quality {q}, display {}, codec {}",
        args.display, args.codec
    );
    info!("WebSocket endpoint: ws://{}/ws", args.ws_addr);

    let (jpeg_tx, _) = broadcast::channel::<bytes::Bytes>(256);
    let (h264_tx, _) = broadcast::channel::<bytes::Bytes>(64);
    let jpeg_tx = Arc::new(jpeg_tx);
    let h264_tx = Arc::new(h264_tx);

    #[cfg(windows)]
    {
        info!("Setting up virtual displays...");
        let mut manager = virtual_display::VirtualDisplayManager::new();
        manager.add_display(1920, 1080, 60);
        if let Err(e) = manager.create_virtual_display(0) {
            warn!("Failed to create virtual display: {}. Using primary display only.", e);
        }
    }

    let cap_jpeg = jpeg_tx.clone();
    std::thread::spawn(move || {
        if let Err(e) = capture::start_capture_pipeline(cap_jpeg) {
            error!("JPEG capture pipeline stopped: {e:?}");
        }
    });

    let cap_h264 = h264_tx.clone();
    std::thread::spawn(move || {
        if let Err(e) = capture::start_h264_pipeline(cap_h264) {
            error!("H.264 capture pipeline stopped: {e:?}");
        }
    });

    let ws_jpeg = jpeg_tx.clone();
    let ws_h264 = h264_tx.clone();
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = ws_server::run(ws_jpeg, Some(ws_h264)).await {
            error!("WebSocket server stopped: {e:?}");
        }
    });

    let quic_handle = if let Some(quic_addr) = args.quic_addr {
        let addr = quic_addr
            .parse()
            .expect("Invalid QUIC listen address format");
        let qh264 = h264_tx.clone();
        info!("QUIC server listening on {}", quic_addr);
        Some(tokio::spawn(async move {
            if let Err(e) = quic_server::run(addr, qh264).await {
                error!("QUIC server stopped: {e:?}");
            }
        }))
    } else {
        None
    };

    info!("Server running. Press Ctrl+C to stop.");
    match signal::ctrl_c().await {
        Ok(()) => info!("Received Ctrl+C, shutting down..."),
        Err(e) => error!("Failed to listen for shutdown signal: {}", e),
    }

    ws_handle.abort();
    if let Some(handle) = quic_handle {
        handle.abort();
    }
    info!("Server stopped");
    Ok(())
}
