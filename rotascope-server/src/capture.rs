use std::io::Cursor;
use anyhow::Result;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use screenshots::Screen;

use crate::virtual_display::{VirtualDisplay, VirtualDisplayManager};

pub struct FrameData {
    pub width: u32,
    pub height: u32,
    pub jpeg_data: Vec<u8>,
}

#[cfg(target_os = "windows")]
pub struct ScreenCapturer {
    // Windows DXGI 捕获实现
}

#[cfg(target_os = "linux")]
pub struct ScreenCapturer {
    // Linux X11 捕获实现
}

#[cfg(target_os = "macos")]
pub struct ScreenCapturer {
    // macOS 捕获实现
}

impl ScreenCapturer {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
    pub fn capture_to_display(display: &mut VirtualDisplay) {
        // Example: capture main screen
        let screen = Screen::from_point(0, 0).unwrap();
        let image = screen.capture().unwrap();

        // Disambiguate the `buffer` call (avoid the `SinkExt::buffer` name clash)
        display.framebuffer =  image.rgba().to_vec();
    }

    pub fn capture_data(&self) -> Result<Vec<u8>> {
        // Example: capture main screen
        let screen = Screen::from_point(0, 0).unwrap();
        let image = screen.capture().unwrap();

        // Disambiguate the `buffer` call (avoid the `SinkExt::buffer` name clash)
        Ok(image.rgba().to_vec())
    }

    // pub async fn capture_display(&self, display_index: u8) -> Result<FrameData> {
    //     // 这里实现具体的屏幕捕获逻辑
    //     // 对于演示，我们创建一个测试图像
    //
    //     let width = 1920;
    //     let height = 1080;
    //
    //     // 创建测试图像 - 实际实现应该捕获真实屏幕
    //     let mut img = ImageBuffer::new(width, height);
    //     // 填充颜色以区分不同的显示器
    //     let color = match display_index {
    //         0 => [255, 0, 0],     // 红色
    //         1 => [0, 255, 0],     // 绿色
    //         2 => [0, 0, 255],     // 蓝色
    //         _ => [128, 128, 128], // 灰色
    //     };
    //
    //     for (_, _, pixel) in img.enumerate_pixels_mut() {
    //         *pixel = Rgba([color[0], color[1], color[2], 255]);
    //     }
    //
    //     // 添加显示器编号文本
    //     self.add_display_text(&mut img, display_index);
    //
    //     // 编码为JPEG
    //     let mut jpeg_data = self.capture_data()?;
    //     // 将 RGBA ImageBuffer 转换为 RGB（去掉 alpha），因为 JPEG 不支持 Rgba8
    //     let rgb_img: image::RgbImage = DynamicImage::ImageRgba8(img).to_rgb8();
    //     let raw = rgb_img.into_raw();
    //     JpegEncoder::new(&mut jpeg_data).encode(
    //         &raw,
    //         width,
    //         height,
    //         image::ColorType::Rgb8.into(),
    //     )?;
    //
    //     Ok(FrameData {
    //         width,
    //         height,
    //         jpeg_data,
    //     })
    // }

    pub async fn capture_display(&self, display_index: u8) -> Result<FrameData> {
        let width = 1920;
        let height = 1080;

        // 创建测试图像 - 实际实现应该捕获真实屏幕
        let img = ImageBuffer::from_vec(width, height,self.capture_data()?).unwrap();
        // 编码为JPEG
        // 使用更高效的 JPEG 编码，降低质量以减少文件大小
        let mut jpeg_data = Vec::new();

        // 将 ImageBuffer 转换为 DynamicImage
        let dynamic_img = image::DynamicImage::ImageRgba8(img);

        // 使用 image crate 的 JPEG 编码，设置质量参数
        dynamic_img.write_to(&mut std::io::Cursor::new(&mut jpeg_data), ImageFormat::Jpeg)?;

        // 如果数据仍然太大，进行二次压缩
        if jpeg_data.len() > 500_000 { // 如果大于 500KB
            jpeg_data = self.compress_jpeg(&jpeg_data, 70)?; // 70% 质量
        }

        log::debug!("Generated JPEG: {}x{}, {} bytes", width, height, jpeg_data.len());
        println!("Generated JPEG: {}x{}, {} bytes", width, height, jpeg_data.len());

        Ok(FrameData {
            width,
            height,
            jpeg_data,
        })
    }


    fn compress_jpeg(&self, original_data: &[u8], quality: u8) -> Result<Vec<u8>> {
        // 解码原始 JPEG 数据
        let img = image::load_from_memory(original_data)?;
        let mut compressed_data = Vec::new();

        // 使用 turbojpeg 或降低分辨率来实现质量调整
        // 这里我们通过缩小图像来实现压缩效果
        let scaled = img.resize(
            (img.width() * quality as u32) / 100,
            (img.height() * quality as u32) / 100,
            image::imageops::FilterType::Lanczos3,
        );

        // 重新编码为 JPEG
        scaled.write_to(&mut Cursor::new(&mut compressed_data), ImageFormat::Jpeg)?;

        Ok(compressed_data)
    }

    fn add_display_text(&self, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, display_index: u8) {
        // 在实际实现中，可以使用 imageproc 添加文本
        // 这里简化为修改一些像素来表示文本
        let text_x = 100;
        let text_y = 100;

        for i in 0..50 {
            for j in 0..50 {
                if i < 10 || i > 40 || j < 10 || j > 40 {
                    let x = text_x + i;
                    let y = text_y + j;
                    if x < img.width() && y < img.height() {
                        img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
impl ScreenCapturer {
    // Windows 特定的捕获实现
    // 使用 DXGI API
}

#[cfg(target_os = "linux")]
impl ScreenCapturer {
    // Linux 特定的捕获实现
    // 使用 X11 或 Wayland API
}
