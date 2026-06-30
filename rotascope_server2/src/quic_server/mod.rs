use time::OffsetDateTime;
use anyhow::{Result, Context};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error, warn};

use crate::capture::ScreenCapturer;
use crate::encoder::VideoEncoder;

pub async fn run(addr: SocketAddr) -> Result<()> {
    // 生成自签名证书（生产环境应使用受信任证书）
 //   let (server_config, server_cert) = configure_quic_server()?;

    let endpoint = Endpoint::server(server_config, addr)?;
    info!("QUIC server listening on {}", addr);

    // 创建屏幕捕获器
    let mut capturer = ScreenCapturer::new(0, 60)?;
    let (width, height) = capturer.resolution();
    let mut encoder = VideoEncoder::new(width, height, 85)?;

    // 接收连接的循环
    while let Some(conn) = endpoint.accept().await {
        let connection = conn.await?;
        info!("Client connected: {}", connection.remote_address());

        // 为每个连接生成任务
        tokio::spawn(handle_client(
            connection,
            capturer.resolution(),
        ));
    }

    Ok(())
}

// fn configure_quic_server() -> Result<(ServerConfig, Vec<u8>)> {
//     use rcgen::{CertificateParams, KeyPair};
//     use std::time::SystemTime;
//
//     let mut cert_params = CertificateParams::default();
//     cert_params.not_before = SystemTime::now().into();
//     cert_params.not_after = OffsetDateTime::from(SystemTime::now() + std::time::Duration::from_secs(365 * 24 * 60 * 60));
//     cert_params.distinguished_name = rcgen::DistinguishedName::new();
//     cert_params.subject_alt_names = vec![ "localhost".into(), "127.0.0.1".into()];
//
//     let key_pair = KeyPair::generate()?;
//     let cert = cert_params.self_signed(&key_pair)?;
//
//     let cert_der = cert?;
//     let priv_key = key_pair.serialize_der();
//
//     let cert_chain = vec![Certificate(cert_der.clone())];
//     let priv_key = PrivateKey(priv_key);
//
//     let mut server_crypto = rustls::ServerConfig::builder()
//         .with_safe_defaults()
//         .with_no_client_auth()
//         .with_single_cert(cert_chain, priv_key)?;
//
//     server_crypto.alpn_protocols = vec![b"screen-mirror".to_vec()];
//
//     let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
//     let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
//     transport_config.max_concurrent_uni_streams(0_u8.into());
//     transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
//
//     Ok((server_config, cert_der))
// }

async fn handle_client(connection: quinn::Connection, resolution: (u32, u32)) -> Result<()> {
    // 发送视频信息
    let video_info = serde_json::json!({
        "type": "video_info",
        "width": resolution.0,
        "height": resolution.1,
        "codec": "h264",
        "fps": 60
    });

    // 打开视频流（单向）
    let mut video_stream = connection.open_uni().await?;
    video_stream.write_all(video_info.to_string().as_bytes()).await?;

    // 创建捕获器和编码器
    let mut capturer = ScreenCapturer::new(0, 60)?;
    let mut encoder = VideoEncoder::new(resolution.0, resolution.1, 85)?;

    // 帧率控制
    let frame_interval = std::time::Duration::from_secs_f64(1.0 / 60.0);
    let mut last_frame = std::time::Instant::now();

    loop {
        // 控制帧率
        let elapsed = last_frame.elapsed();
        if elapsed < frame_interval {
            tokio::time::sleep(frame_interval - elapsed).await;
        }

        // 捕获帧
        if let Some(frame) = capturer.capture()? {
            // 编码
            match encoder.encode_frame(&frame) {
                Ok(encoded) => {
                    // 发送
                    if let Err(e) = video_stream.write_all(&encoded).await {
                        error!("Failed to send frame: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to encode frame: {}", e);
                }
            }
        }

        last_frame = std::time::Instant::now();

        // 检查连接状态
        if let Err(e) = connection.stats() {
            error!("Connection lost: {}", e);
            break;
        }
    }

    info!("Client disconnected");
    Ok(())
}