use anyhow::{Context, Result};
use quinn::{Connection, Endpoint, ServerConfig, TransportConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use crate::capture::ScreenCapturer;
use crate::encoder::VideoEncoder;

pub async fn run(addr: SocketAddr) -> Result<()> {
    let (server_config, _server_cert) = configure_quic_server()?;

    let endpoint = Endpoint::server(server_config, addr)?;
    info!("QUIC server listening on {}", addr);

    let capturer = ScreenCapturer::new(0, 60)?;
    let (width, height) = capturer.resolution();
    let _encoder = VideoEncoder::new(width, height, 85)?;

    while let Some(conn) = endpoint.accept().await {
        match conn.await {
            Ok(connection) => {
                info!("Client connected: {}", connection.remote_address());
                let (w, h) = capturer.resolution();
                tokio::spawn(handle_client(connection, (w, h)));
            }
            Err(e) => {
                warn!("Connection rejected: {}", e);
            }
        }
    }

    Ok(())
}

fn configure_quic_server() -> Result<(ServerConfig, Vec<u8>)> {
    use rcgen::{CertificateParams, KeyPair};
    use std::time::SystemTime;

    let mut cert_params = CertificateParams::default();
    cert_params.not_before = SystemTime::now().into();
    cert_params.not_after = (SystemTime::now() + Duration::from_secs(365 * 24 * 60 * 60)).into();
    cert_params.distinguished_name = rcgen::DistinguishedName::new();
    cert_params.subject_alt_names = vec![
        rcgen::SanType::DnsName("localhost".try_into().unwrap()),
        rcgen::SanType::IpAddress("127.0.0.1".parse().unwrap()),
    ];

    let key_pair = KeyPair::generate()?;
    let cert = cert_params.self_signed(&key_pair)?;

    let cert_der = cert.der().to_vec();
    let priv_key = key_pair.serialize_der();

    let cert_chain = vec![rustls::Certificate(cert_der.clone())];
    let priv_key = rustls::PrivateKey(priv_key);

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, priv_key)?;

    server_crypto.alpn_protocols = vec![b"rotascope-screen".to_vec()];

    let mut transport_config = TransportConfig::default();
    transport_config.max_concurrent_uni_streams(0_u8.into());
    transport_config.keep_alive_interval(Some(Duration::from_secs(5)));

    let mut server_config = ServerConfig::with_crypto(Arc::new(server_crypto));
    server_config.transport_config(Arc::new(transport_config));

    Ok((server_config, cert_der))
}

async fn handle_client(connection: Connection, resolution: (u32, u32)) -> Result<()> {
    let video_info = serde_json::json!({
        "type": "video_info",
        "width": resolution.0,
        "height": resolution.1,
        "codec": "h264",
        "fps": 60
    });

    let mut video_stream = connection.open_uni().await?;
    video_stream
        .write_all(video_info.to_string().as_bytes())
        .await?;

    let mut capturer = ScreenCapturer::new(0, 60)?;
    let mut encoder = VideoEncoder::new(resolution.0, resolution.1, 85)?;

    let frame_interval = Duration::from_secs_f64(1.0 / 60.0);
    let mut last_frame = Instant::now();

    loop {
        let elapsed = last_frame.elapsed();
        if elapsed < frame_interval {
            tokio::time::sleep(frame_interval - elapsed).await;
        }

        if let Some(frame) = capturer.capture()? {
            match encoder.encode_frame(&frame) {
                Ok(encoded) => {
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

        last_frame = Instant::now();

        if connection.close_reason().is_some() {
            warn!("Connection closed by peer");
            break;
        }
    }

    info!("Client disconnected");
    Ok(())
}
