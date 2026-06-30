use anyhow::{Result, Context};
use std::time::{Duration, Instant};
use tracing::error;
#[cfg(windows)]
use windows::Win32::Graphics::{
    Dxgi::{
        IDXGIOutputDuplication, IDXGIFactory1, IDXGIAdapter1, IDXGIOutput,
        DXGI_OUTDUPL_FRAME_INFO, DXGI_OUTPUT_DESC,
        DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_WAIT_TIMEOUT
    },
    Direct3D11::{
        ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
        D3D11CreateDevice, D3D11_CPU_ACCESS_READ, D3D11_USAGE_STAGING,
        D3D11_FENCE_FLAG_NONE, D3D11_TEX2D_DSV, D3D11_MAPPED_SUBRESOURCE
    },
    Direct3D::D3D_DRIVER_TYPE_HARDWARE
};
use windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE2D_DESC;
use windows::Win32::Graphics::Dxgi::IDXGIAdapter;

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

pub struct ScreenCapturer {
    #[cfg(windows)]
    dupl: IDXGIOutputDuplication,
    #[cfg(windows)]
    device: ID3D11Device,
    #[cfg(windows)]
    context: ID3D11DeviceContext,
    width: u32,
    height: u32,
    fps: u32,
    last_frame: Instant,
}

impl ScreenCapturer {
    pub fn new(display_idx: usize, fps: u32) -> Result<Self> {
        #[cfg(not(windows))]
        {
            anyhow::bail!("Screen capture only supported on Windows in this version");
        }

        #[cfg(windows)]
        {
            use windows::core::Interface;
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

            unsafe {
                // 初始化COM
                CoInitializeEx(None, COINIT_MULTITHREADED);

                // 创建D3D11设备
                let mut device: Option<ID3D11Device> = None;
                let mut context: Option<ID3D11DeviceContext> = None;

                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_HARDWARE,
                    None,
                    0,
                    None,
                    0,
                    7, // D3D11_SDK_VERSION
                    Some(&mut device),
                    Some(&mut context),
                    None
                )?;

                let device = device.unwrap();
                let context = context.unwrap();

                // 获取DXGI工厂
                let mut dxgi_device: windows::Win32::Graphics::Dxgi::IDXGIDevice = device.cast()?;
                let adapter: IDXGIAdapter = dxgi_device.GetAdapter()?;
                let factory: IDXGIFactory1 = adapter.GetParent()?;

                // 获取指定的显示输出
                let output: IDXGIOutput = adapter.EnumOutputs(display_idx as u32)?;
                let output1: windows::Win32::Graphics::Dxgi::IDXGIOutput1 = output.cast()?;

                // 创建桌面复制接口
                let mut dupl: Option<IDXGIOutputDuplication> = None;
                output1.DuplicateOutput(&device)?;
                let dupl = dupl.unwrap();

                // 获取显示信息
                let mut output_desc: DXGI_OUTPUT_DESC = std::mem::zeroed();
                output.GetDesc()?;
                let width = output_desc.DesktopCoordinates.right - output_desc.DesktopCoordinates.left;
                let height = output_desc.DesktopCoordinates.bottom - output_desc.DesktopCoordinates.top;

                Ok(Self {
                    dupl,
                    device,
                    context,
                    width: width as u32,
                    height: height as u32,
                    fps,
                    last_frame: Instant::now(),
                })
            }
        }
    }

    pub fn capture(&mut self) -> Result<Option<Frame>> {
        let frame_time = Duration::from_secs_f64(1.0 / self.fps as f64);
        let elapsed = self.last_frame.elapsed();

        if elapsed < frame_time {
            return Ok(None);
        }

        #[cfg(not(windows))]
        {
            return Ok(None);
        }

        #[cfg(windows)]
        {
            use windows::core::Interface;

            unsafe {
                let mut frame_info: DXGI_OUTDUPL_FRAME_INFO = std::mem::zeroed();
                let mut desktop_resource: Option<ID3D11Texture2D> = None;

                // 尝试获取下一帧
                match self.dupl.AcquireNextFrame(16, &mut frame_info, &mut desktop_resource) {
                    Ok(_) => {
                        let resource = desktop_resource.unwrap();

                        // 创建暂存纹理用于CPU读取
                        let desc = D3D11_TEXTURE2D_DESC {
                            Width: self.width,
                            Height: self.height,
                            MipLevels: 1,
                            ArraySize: 1,
                            Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
                            SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
                                Count: 1,
                                Quality: 0,
                            },
                            Usage: D3D11_USAGE_STAGING,
                            BindFlags: D3D11_FENCE_FLAG_NONE.0 as u32,
                            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
                            MiscFlags: 0,
                        };

                        let mut staging_texture: Option<ID3D11Texture2D> = None;
                        self.device.CreateTexture2D(&desc, None, Some(&mut staging_texture))?;
                        let staging_texture = staging_texture.unwrap();

                        // 复制到暂存纹理
                        self.context.CopyResource(
                            staging_texture.cast()?,
                            resource.cast()?
                        );

                        // 映射到CPU内存
                        let mut mapped: D3D11_MAPPED_SUBRESOURCE = std::mem::zeroed();
                        self.context.Map(
                            staging_texture.cast()?,
                            0,
                            windows::Win32::Graphics::Direct3D11::D3D11_MAP_READ,
                            0,
                            Some(&mut mapped)
                        )?;

                        // 复制数据
                        let row_pitch = mapped.RowPitch as usize;
                        let src_data = std::slice::from_raw_parts(
                            mapped.pData as *const u8,
                            row_pitch * self.height as usize
                        );

                        let mut frame_data = Vec::with_capacity((self.width * self.height * 4) as usize);
                        for y in 0..self.height as usize {
                            let start = y * row_pitch;
                            let end = start + (self.width * 4) as usize;
                            frame_data.extend_from_slice(&src_data[start..end]);
                        }

                        // 解映射
                        self.context.Unmap(staging_texture.cast()?, 0);

                        // 释放帧
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
                    Err(e) if e.code() == DXGI_ERROR_WAIT_TIMEOUT => {
                        Ok(None)
                    }
                    Err(e) => {
                        error!("Capture error: {:?}", e);
                        Err(e.into())
                    }
                }
            }
        }
    }

    pub fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}