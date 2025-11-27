mod capture;
mod virtual_display;
mod server;
mod CrossPlatformCapturer;
mod ScreenCapturer;

use env_logger::Env;
use server::*;
use std::sync::Arc;
use rotascope_core::Result;


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    log::info!("Starting Multi-Display PC Server...");
    println!("Starting Multi-Display PC Server...");

    let server = Arc::new(MultiDisplayServer::new(3)?); // 3个虚拟显示器

    // 启动虚拟显示器
    server.start_virtual_displays().await?;

    // 启动网络服务器
    server.start_server("0.0.0.0:8080").await?;

    Ok(())
}