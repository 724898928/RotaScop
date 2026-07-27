use std::time::{Duration, Instant};
use std::sync::Arc;

use anyhow::Result;
use bytes::Bytes;
use image::codecs::jpeg::JpegEncoder;
use image::RgbaImage;
use scrap::{Capturer, Display};
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, info, trace, warn};

pub struct RawFrame {
    pub bgra: Vec<u8>,
    pub width: usize,
    pub height: usize,
    pub seq: u64,
}

pub struct Frame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub timestamp: u64,
    pub format: PixelFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    Rgba,
    Bgra,
    Nv12,
}

const TARGET_FPS: u64 = 60;
const DEFAULT_JPEG_QUALITY: u8 = 40;

pub fn start_capture_pipeline(tx: Arc<broadcast::Sender<Bytes>>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    let quality = std::env::var("ROTASCOPE_QUALITY")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        .unwrap_or(DEFAULT_JPEG_QUALITY);

    info!("Capture pipeline: target {TARGET_FPS}fps, JPEG quality {quality}");
    rt.block_on(pipeline(tx, quality))
}

async fn pipeline(tx: Arc<broadcast::Sender<Bytes>>, quality: u8) -> Result<()> {
    let frame_duration = Duration::from_micros(1_000_000 / TARGET_FPS as u64);
    let (mut capturer, width, height) = select_monitor()?;

    let (raw_tx, mut raw_rx) = mpsc::channel::<RawFrame>(2);
    let mut seq: u64 = 0;

    let encode_tx = tx.clone();
    let _encode_task = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        while let Some(mut raw) = rt.block_on(raw_rx.recv()) {
            let start = Instant::now();

            bgra_to_rgba_inplace(&mut raw.bgra);

            let image = match RgbaImage::from_raw(
                raw.width as u32,
                raw.height as u32,
                raw.bgra,
            ) {
                Some(img) => img,
                None => {
                    error!("failed to create RgbaImage from raw data");
                    continue;
                }
            };

            let mut jpeg_bytes = Vec::with_capacity(100_000);
            let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, quality);
            if let Err(e) = encoder.encode_image(&image) {
                error!("JPEG encode error: {e:?}");
                continue;
            }

            let elapsed = start.elapsed();
            trace!("encode frame {} took {}ms", raw.seq, elapsed.as_millis());

            let _ = encode_tx.send(Bytes::from(jpeg_bytes));
        }
    });

    loop {
        if tx.receiver_count() == 0 && raw_tx.capacity() > 0 {
            sleep(Duration::from_millis(250)).await;
            continue;
        }

        let start = Instant::now();

        match capture_raw(&mut capturer, width, height) {
            Some(mut raw) => {
                seq += 1;
                raw.seq = seq;
                if raw_tx.try_send(raw).is_err() {
                    trace!("encode backlog, dropping frame {}", seq);
                }
            }
            None => {
                sleep(Duration::from_millis(1)).await;
                continue;
            }
        }

        let elapsed = start.elapsed();
        if elapsed < frame_duration {
            sleep(frame_duration - elapsed).await;
        }
    }
}

fn bgra_to_rgba_inplace(data: &mut [u8]) {
    for pixel in data.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
}

fn select_monitor() -> Result<(Capturer, usize, usize)> {
    let selected_index = std::env::var("ROTASCOPE_DISPLAY_INDEX")
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
        warn!(
            "ROTASCOPE_DISPLAY_INDEX={selected_index} out of range; using display 0 of {count}"
        );
        0
    } else {
        selected_index
    };

    let display = displays
        .into_iter()
        .nth(index)
        .ok_or_else(|| anyhow::anyhow!("selected display not found"))?;

    let width = display.width();
    let height = display.height();
    let capturer = Capturer::new(display)
        .map_err(|e| anyhow::anyhow!("failed to create capturer: {e}"))?;

    info!("Capturing display index {index}: {width}x{height}");
    Ok((capturer, width, height))
}

fn capture_raw(capturer: &mut Capturer, width: usize, height: usize) -> Option<RawFrame> {
    use std::io::ErrorKind::WouldBlock;

    loop {
        match capturer.frame() {
            Ok(buffer) => {
                return Some(RawFrame {
                    bgra: buffer.to_vec(),
                    width,
                    height,
                    seq: 0,
                });
            }
            Err(ref e) if e.kind() == WouldBlock => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            Err(e) => {
                error!("capture failed: {e:?}");
                return None;
            }
        }
    }
}

pub fn start_h264_pipeline(tx: Arc<broadcast::Sender<Bytes>>) -> Result<()> {
    use crate::encoder::VideoEncoder;

    let display_idx = std::env::var("ROTASCOPE_DISPLAY_INDEX")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    let mut capturer = ScreenCapturer::new(display_idx, 60)?;
    let (width, height) = capturer.resolution();
    let mut encoder = VideoEncoder::new(width, height, 85)?;

    info!("H.264 pipeline started: {width}x{height} @ 60fps");
    let frame_interval = std::time::Duration::from_micros(1_000_000 / 60);

    loop {
        let start = std::time::Instant::now();

        if tx.receiver_count() == 0 {
            std::thread::sleep(std::time::Duration::from_millis(250));
            continue;
        }

        match capturer.capture() {
            Ok(Some(frame)) => {
                match encoder.encode_frame(&frame) {
                    Ok(encoded) => {
                        let _ = tx.send(Bytes::from(encoded));
                    }
                    Err(e) => {
                        trace!("H.264 encode error: {e:?}");
                    }
                }
            }
            Ok(None) => {
                std::thread::sleep(std::time::Duration::from_millis(1));
                continue;
            }
            Err(e) => {
                error!("H.264 capture error: {e:?}");
                break;
            }
        }

        let elapsed = start.elapsed();
        if elapsed < frame_interval {
            std::thread::sleep(frame_interval - elapsed);
        }
    }

    Ok(())
}

#[cfg(windows)]
pub struct ScreenCapturer {
    device: windows::Win32::Graphics::Direct3D11::ID3D11Device,
    context: windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
    dupl: windows::Win32::Graphics::Dxgi::IDXGIOutputDuplication,
    width: u32,
    height: u32,
    fps: u32,
    last_frame: Instant,
}

#[cfg(not(windows))]
pub struct ScreenCapturer {
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
}

impl ScreenCapturer {
    #[cfg(windows)]
    pub fn new(display_idx: usize, fps: u32) -> Result<Self> {
        use windows::core::Interface;
        use windows::Win32::Foundation::HMODULE;
        use windows::Win32::Graphics::{
            Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL},
            Direct3D11::{
                D3D11CreateDevice, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, ID3D11DeviceContext,
            },
            Dxgi::{IDXGIAdapter, IDXGIDevice, IDXGIFactory1, IDXGIOutput, IDXGIOutput1},
        };
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let mut device: Option<windows::Win32::Graphics::Direct3D11::ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;

            D3D11CreateDevice(
                None as Option<&IDXGIAdapter>,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_FLAG(0),
                None as Option<&[D3D_FEATURE_LEVEL]>,
                D3D11_SDK_VERSION,
                Some(&mut device as *mut Option<windows::Win32::Graphics::Direct3D11::ID3D11Device>),
                None as Option<*mut D3D_FEATURE_LEVEL>,
                Some(&mut context as *mut Option<ID3D11DeviceContext>),
            )?;

            let device = device.unwrap();
            let context = context.unwrap();

            let dxgi_device: IDXGIDevice = device.cast()?;
            let adapter: IDXGIAdapter = dxgi_device.GetAdapter()?;
            let _factory: IDXGIFactory1 = adapter.GetParent()?;

            let output: IDXGIOutput = adapter.EnumOutputs(display_idx as u32)?;
            let output1: IDXGIOutput1 = output.cast()?;

            let dupl = output1.DuplicateOutput(&device)?;

            let output_desc = output.GetDesc()?;
            let width = output_desc.DesktopCoordinates.right - output_desc.DesktopCoordinates.left;
            let height = output_desc.DesktopCoordinates.bottom - output_desc.DesktopCoordinates.top;

            Ok(Self {
                device,
                context,
                dupl,
                width: width as u32,
                height: height as u32,
                fps,
                last_frame: Instant::now(),
            })
        }
    }

    #[cfg(not(windows))]
    pub fn new(_display_idx: usize, _fps: u32) -> Result<Self> {
        anyhow::bail!("Screen capture is only supported on Windows in this version");
    }

    #[cfg(windows)]
    pub fn capture(&mut self) -> Result<Option<Frame>> {
        use windows::core::Interface;
        use windows::Win32::Graphics::{
            Direct3D11::{
                D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE,
                D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
            },
            Dxgi::{
                IDXGIResource, DXGI_OUTDUPL_FRAME_INFO, DXGI_ERROR_ACCESS_LOST,
                DXGI_ERROR_WAIT_TIMEOUT,
            },
            Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
        };

        unsafe {
            let mut frame_info: DXGI_OUTDUPL_FRAME_INFO = std::mem::zeroed();
            let mut desktop_resource: Option<IDXGIResource> = None;

            match self.dupl.AcquireNextFrame(8, &mut frame_info, &mut desktop_resource) {
                Ok(()) => {}
                Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => return Ok(None),
                Err(e) => {
                    if e.code() == DXGI_ERROR_ACCESS_LOST {
                        warn!("DXGI access lost, need to recreate capturer");
                    }
                    error!("Capture error: {:?}", e);
                    return Err(e.into());
                }
            }

            let resource = desktop_resource.unwrap();
            let desktop_texture: windows::Win32::Graphics::Direct3D11::ID3D11Texture2D =
                resource.cast()?;

            let desc = D3D11_TEXTURE2D_DESC {
                Width: self.width,
                Height: self.height,
                MipLevels: 1,
                ArraySize: 1,
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_STAGING,
                BindFlags: 0,
                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                MiscFlags: 0,
            };

            let mut staging_texture: Option<
                windows::Win32::Graphics::Direct3D11::ID3D11Texture2D,
            > = None;
            self.device.CreateTexture2D(&desc, None, Some(&mut staging_texture))?;
            let staging_texture = staging_texture.unwrap();

            self.context.CopyResource(&staging_texture, &desktop_texture);

            let mut mapped: D3D11_MAPPED_SUBRESOURCE = std::mem::zeroed();
            self.context
                .Map(&staging_texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;

            let row_pitch = mapped.RowPitch as usize;
            let src_data =
                std::slice::from_raw_parts(mapped.pData as *const u8, row_pitch * self.height as usize);

            let mut frame_data = Vec::with_capacity((self.width * self.height * 4) as usize);
            for y in 0..self.height as usize {
                let start = y * row_pitch;
                let end = start + (self.width * 4) as usize;
                frame_data.extend_from_slice(&src_data[start..end]);
            }

            self.context.Unmap(&staging_texture, 0);
            self.dupl.ReleaseFrame()?;

            self.last_frame = Instant::now();

            Ok(Some(Frame {
                data: frame_data,
                width: self.width,
                height: self.height,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                format: PixelFormat::Bgra,
            }))
        }
    }

    #[cfg(not(windows))]
    pub fn capture(&mut self) -> Result<Option<Frame>> {
        Ok(None)
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
