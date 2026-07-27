use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;
use quinn::{Connection, Endpoint, ServerConfig, TransportConfig};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub async fn run(
    addr: std::net::SocketAddr,
    h264_tx: Arc<broadcast::Sender<Bytes>>,
) -> Result<()> {
    let (server_config, _server_cert) = configure_quic_server()?;

    let endpoint = Endpoint::server(server_config, addr)?;
    info!("QUIC server listening on {}", addr);

    while let Some(conn) = endpoint.accept().await {
        match conn.await {
            Ok(connection) => {
                info!("QUIC client connected: {}", connection.remote_address());
                let rx = h264_tx.subscribe();
                tokio::spawn(handle_client(connection, rx));
            }
            Err(e) => {
                warn!("QUIC connection rejected: {}", e);
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
    cert_params.not_after =
        (SystemTime::now() + Duration::from_secs(365 * 24 * 60 * 60)).into();
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

async fn handle_client(
    connection: Connection,
    mut rx: broadcast::Receiver<Bytes>,
) -> Result<()> {
    let video_info = serde_json::json!({
        "type": "video_info",
        "codec": "h264",
        "fps": 60
    });

    let mut stream = connection.open_uni().await?;
    stream
        .write_all(video_info.to_string().as_bytes())
        .await?;

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(frame) => {
                        if let Err(e) = stream.write_all(&frame).await {
                            error!("QUIC send error: {e:?}");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("QUIC dropped {n} frames");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                if connection.close_reason().is_some() {
                    warn!("QUIC connection closed by peer");
                    break;
                }
            }
        }
    }

    info!("QUIC client disconnected");
    Ok(())
}
