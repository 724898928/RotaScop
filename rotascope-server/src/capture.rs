use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use bytes::Bytes;
use image::{codecs::jpeg::JpegEncoder, RgbaImage};
use log::{error, info, warn};
use scrap::{Capturer, Display};
use tokio::sync::broadcast;
use tokio::time::sleep;

const TARGET_FPS: u64 = 15;

pub async fn capture_loop(tx: Arc<broadcast::Sender<Bytes>>) -> Result<()> {
    let frame_duration = Duration::from_millis(1000 / TARGET_FPS);
    let (mut capturer, width, height) = select_monitor()?;

    loop {
        if tx.receiver_count() == 0 {
            sleep(Duration::from_millis(250)).await;
            continue;
        }

        let start = Instant::now();

        match capture_frame(&mut capturer, width, height) {
            Ok(jpeg_bytes) => {
                let _ = tx.send(Bytes::from(jpeg_bytes));
            }
            Err(e) => {
                error!("capture error: {e:?}");
                sleep(Duration::from_millis(100)).await;
            }
        }

        let elapsed = start.elapsed();
        if elapsed < frame_duration {
            sleep(frame_duration - elapsed).await;
        }
    }
}

fn select_monitor() -> Result<(Capturer, usize, usize)> {
    let selected_index = env::var("ROTASCOPE_DISPLAY_INDEX")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    let displays = Display::all()
        .map_err(|e| anyhow::anyhow!("failed to enumerate displays: {e}"))?;

    if displays.is_empty() {
        anyhow::bail!("no displays found");
    }

    let count = displays.len();
    let index = if selected_index >= count {
        warn!("ROTASCOPE_DISPLAY_INDEX={selected_index} is out of range; using display 0 of {count}");
        0
    } else {
        selected_index
    };

    let display = displays
        .into_iter()
        .nth(index)
        .context("selected display not found")?;

    let width = display.width();
    let height = display.height();

    let capturer = Capturer::new(display)
        .map_err(|e| anyhow::anyhow!("failed to create capturer: {e}"))?;

    info!("Capturing display index {index}: {width}x{height}");
    Ok((capturer, width, height))
}

fn capture_frame(capturer: &mut Capturer, width: usize, height: usize) -> Result<Vec<u8>> {
    use std::io::ErrorKind::WouldBlock;

    loop {
        match capturer.frame() {
            Ok(buffer) => {
                let mut rgba = Vec::with_capacity(width * height * 4);

                // BGRA -> RGBA conversion
                for chunk in buffer.chunks(4) {
                    if chunk.len() >= 3 {
                        rgba.push(chunk[2]); // R
                        rgba.push(chunk[1]); // G
                        rgba.push(chunk[0]); // B
                        rgba.push(255);      // A
                    }
                }

                let Some(image) = RgbaImage::from_raw(width as u32, height as u32, rgba) else {
                    anyhow::bail!("failed to create image from captured data");
                };

                let mut jpeg_bytes = Vec::with_capacity(200_000);
                let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, 72);
                encoder.encode_image(&image)?;

                return Ok(jpeg_bytes);
            }
            Err(ref e) if e.kind() == WouldBlock => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => {
                anyhow::bail!("capture failed: {e}");
            }
        }
    }
}
