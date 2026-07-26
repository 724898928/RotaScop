use anyhow::Result;

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
        use std::mem;
        use windows::core::Interface;
        use windows::Win32::Graphics::{
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::{
                D3D11CreateDevice, D3D11_CPU_ACCESS_READ, D3D11_USAGE_STAGING,
                D3D11_FENCE_FLAG_NONE,
            },
            Dxgi::{
                IDXGIAdapter, IDXGIFactory1, IDXGIOutput, IDXGIOutput1,
                DXGI_OUTPUT_DESC,
            },
            Dxgi::Common::DXGI_SAMPLE_DESC,
        };
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok();

            let mut device: Option<windows::Win32::Graphics::Direct3D11::ID3D11Device> = None;
            let mut context: Option<windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext> = None;

            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                0,
                None,
                0,
                7,
                Some(&mut device),
                Some(&mut context),
                None,
            )?;

            let device = device.unwrap();
            let context = context.unwrap();

            let dxgi_device: windows::Win32::Graphics::Dxgi::IDXGIDevice = device.cast()?;
            let adapter: IDXGIAdapter = dxgi_device.GetAdapter()?;
            let _factory: IDXGIFactory1 = adapter.GetParent()?;

            let output: IDXGIOutput = adapter.EnumOutputs(display_idx as u32)?;
            let output1: IDXGIOutput1 = output.cast()?;

            let dupl = output1.DuplicateOutput(&device)?;

            let mut output_desc: DXGI_OUTPUT_DESC = mem::zeroed();
            output.GetDesc(&mut output_desc)?;
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

}
    }

    #[cfg(windows)]
    pub fn capture(&mut self) -> Result<Option<Frame>> {
        use windows::core::Interface;
        use windows::Win32::Graphics::{
            Direct3D11::{
                D3D11_CPU_ACCESS_READ, D3D11_MAP_READ, D3D11_MAPPED_SUBRESOURCE,
                D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING, D3D11_FENCE_FLAG_NONE,
            },
            Dxgi::{
                DXGI_OUTDUPL_FRAME_INFO, DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_WAIT_TIMEOUT,
            },
            Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
        };

        let frame_time = Duration::from_secs_f64(1.0 / self.fps as f64);
        if self.last_frame.elapsed() < frame_time {
            return Ok(None);
        }

        unsafe {
            let mut frame_info: DXGI_OUTDUPL_FRAME_INFO = std::mem::zeroed();
            let mut desktop_resource: Option<windows::Win32::Graphics::Direct3D11::ID3D11Texture2D> = None;

            match self.dupl.AcquireNextFrame(16, &mut frame_info, &mut desktop_resource) {
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
                BindFlags: D3D11_FENCE_FLAG_NONE.0 as u32,
                CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                MiscFlags: 0,
            };

            let mut staging_texture: Option<windows::Win32::Graphics::Direct3D11::ID3D11Texture2D> = None;
            self.device.CreateTexture2D(&desc, None, Some(&mut staging_texture))?;
            let staging_texture = staging_texture.unwrap();

            self.context.CopyResource(
                &staging_texture.cast()?,
                &resource.cast()?,
            );

            let mut mapped: D3D11_MAPPED_SUBRESOURCE = std::mem::zeroed();
            self.context.Map(
                &staging_texture.cast()?,
                0,
                D3D11_MAP_READ,
                0,
                Some(&mut mapped),
            )?;

            let row_pitch = mapped.RowPitch as usize;
            let src_data = std::slice::from_raw_parts(
                mapped.pData as *const u8,
                row_pitch * self.height as usize,
            );

            let mut frame_data = Vec::with_capacity((self.width * self.height * 4) as usize);
            for y in 0..self.height as usize {
                let start = y * row_pitch;
                let end = start + (self.width * 4) as usize;
                frame_data.extend_from_slice(&src_data[start..end]);
            }

            self.context.Unmap(&staging_texture.cast()?, 0);
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
