mod capture;
mod encoder;
mod quic_server;
mod input_injector;
mod virtual_display;
mod utils;

use anyhow::Result;
use clap::Parser;
use tokio::signal;
use tracing::{info, error, warn};
use windows::core::PCWSTR;

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
    info!("Starting screen mirror server on {}", args.listen_addr);

    // 创建虚拟显示器（Windows）
    #[cfg(windows)]
    {
        info!("Creating virtual display...");
        if let Err(e) = create_virtual_display() {
            warn!("Failed to create virtual display: {}. Using primary display.", e);
        }
    }

    // 2. 启动QUIC服务器
    let server_handle = tokio::spawn(quic_server::run(args.listen_addr.parse()?));

    //  捕获键盘中断信号 等待终止信号
    match signal::ctrl_c().await {
        Ok(()) => info!("Received CTRL+C, shutting down..."),
        Err(err) => error!("Failed to listen for shutdown signal: {}", err),
    }
    info!("Shutting down...");
    server_handle.abort();
    info!("Server stopped");
    Ok(())
}
#[cfg(windows)]
fn create_virtual_display() -> Result<()> {
    use windows::Win32::Graphics::Gdi::{
        DEVMODEW, DISPLAY_DEVICEW, ChangeDisplaySettingsExW,
        CDS_UPDATEREGISTRY, CDS_NORESET, DISP_CHANGE_SUCCESSFUL
    };
    use std::ptr;
    use std::mem::zeroed;

    unsafe {
        let mut devmode: DEVMODEW = zeroed();
        devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        devmode.dmDriverExtra = 0;
        devmode.dmFields = windows::Win32::Graphics::Gdi::DEVMODE_FIELD_FLAGS(0x00800000); // DM_POSITION
        devmode.dmPelsWidth = 1920;
        devmode.dmPelsHeight = 1080;
        devmode.Anonymous1.Anonymous2.dmPosition.x = 0;
        devmode.Anonymous1.Anonymous2.dmPosition.y = 0;
        devmode.dmDisplayFrequency = 60;

        let device_name = wide_string("\\\\.\\DISPLAY2");

        let result = ChangeDisplaySettingsExW(
            PCWSTR(device_name.as_ptr()),
            Some(&devmode),
            None,
            CDS_UPDATEREGISTRY | CDS_NORESET,
            Some(ptr::null_mut())
        );

        if result == DISP_CHANGE_SUCCESSFUL {
            info!("Virtual display created successfully");
            Ok(())
        } else {
            anyhow::bail!("Failed to create virtual display: error code {}", result.0)
        }
    }
}

#[cfg(windows)]
fn wide_string(s: &str) -> Vec<u16> {
    use std::os::windows::prelude::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}