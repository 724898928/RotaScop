mod capture;
mod server;

use std::sync::Arc;

use anyhow::Result;
use log::info;
use tokio::sync::broadcast;
use warp::Filter;

const DEFAULT_PORT: u16 = 8083;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let (tx, _rx) = broadcast::channel::<bytes::Bytes>(16);
    let tx = Arc::new(tx);

    let capture_tx = tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(async move {
            if let Err(e) = capture::capture_loop(capture_tx).await {
                log::error!("capture loop stopped: {e:?}");
            }
        });
    });

    let addr = ([0, 0, 0, 0], DEFAULT_PORT);
    info!("RotaScope server listening on ws://0.0.0.0:{DEFAULT_PORT}/ws");
    warp::serve(server::ws_route(tx).with(warp::cors().allow_any_origin()))
        .run(addr)
        .await;

    Ok(())
}
