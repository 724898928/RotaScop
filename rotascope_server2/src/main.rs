mod capture;
mod encoder;
mod input_injector;
mod quic_server;
mod utils;
mod virtual_display;

use anyhow::Result;
use clap::Parser;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 服务器监听地址
    #[arg(short, long, default_value = "0.0.0.0:1234")]
    listen_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    info!("Starting RotaScope server v2 on {}", args.listen_addr);

    let addr: SocketAddr = args
        .listen_addr
        .parse()
        .expect("Invalid listen address format");

    // Create virtual displays (Windows only)
    #[cfg(windows)]
    {
        info!("Setting up virtual displays...");
        let mut manager = virtual_display::VirtualDisplayManager::new();
        manager.add_display(1920, 1080, 60);
        if let Err(e) = manager.create_virtual_display(0) {
            warn!("Failed to create virtual display: {}. Using primary display only.", e);
        }
    }

    // Start QUIC server
    let server_handle = tokio::spawn(quic_server::run(addr));

    info!("Server running. Press Ctrl+C to stop.");
    match signal::ctrl_c().await {
        Ok(()) => info!("Received Ctrl+C, shutting down..."),
        Err(e) => error!("Failed to listen for shutdown signal: {}", e),
    }

    server_handle.abort();
    info!("Server stopped");
    Ok(())
}
