use anyhow::Result;
use image::{ImageBuffer, Rgba, ImageFormat, DynamicImage};
use screenshots::Screen;
use std::io::Cursor;
use actix::ActorTryFutureExt;
use image::codecs::jpeg::JpegEncoder;

pub struct FrameData {
    pub width: u32,
    pub height: u32,
    pub jpeg_data: Vec<u8>,
}

pub struct ScreenCapturer;

impl ScreenCapturer {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
    pub fn capture_to_display(display: &mut FrameData) {
        // Example: capture main screen
        let screen = Screen::from_point(0, 0).unwrap();
        let image = screen.capture().unwrap();

        // Disambiguate the `buffer` call (avoid the `SinkExt::buffer` name clash)
        display.jpeg_data =  image.rgba().to_vec();
    }
    pub fn capture_data(&self) -> Result<Vec<u8>> {
        // Example: capture main screen
        let screen = Screen::from_point(0, 0).unwrap();
        let image = screen.capture().unwrap();

        // Disambiguate the `buffer` call (avoid the `SinkExt::buffer` name clash)
        Ok(image.rgba().to_vec())
    }

    // pub async fn capture_display(&self, display_index: u8) -> Result<FrameData> {
    //     // 使用更小的分辨率以减少数据量
    //     let width = 1280;
    //     let height = 720;
    //
    //     let mut img = ImageBuffer::new(width, height);
    //
    //     // 根据显示器索引填充不同颜色
    //     let (bg_color, text_color) = match display_index {
    //         0 => ([41, 99, 235], [255, 255, 255]), // 蓝色背景
    //         1 => ([22, 163, 74], [255, 255, 255]),  // 绿色背景
    //         2 => ([220, 38, 38], [255, 255, 255]),  // 红色背景
    //         _ => ([107, 114, 128], [255, 255, 255]), // 灰色背景
    //     };
    //
    //     // 填充背景
    //     for pixel in img.pixels_mut() {
    //         *pixel = Rgba([bg_color[0], bg_color[1], bg_color[2], 255]);
    //     }
    //
    //     // 添加显示器信息文本
    //     self.draw_display_info(&mut img, display_index, text_color);
    //
    //     // 使用更高效的 JPEG 编码，降低质量以减少文件大小
    //     let mut jpeg_data = Vec::new();
    //
    //     // 将 ImageBuffer 转换为 DynamicImage
    //     let dynamic_img = image::DynamicImage::ImageRgba8(img);
    //
    //     // 使用 image crate 的 JPEG 编码，设置质量参数
    //     dynamic_img.write_to(&mut std::io::Cursor::new(&mut jpeg_data), ImageFormat::Jpeg)?;
    //
    //     // 如果数据仍然太大，进行二次压缩
    //     if jpeg_data.len() > 500_000 { // 如果大于 500KB
    //         jpeg_data = self.compress_jpeg(&jpeg_data, 70)?; // 70% 质量
    //     }
    //
    //     log::debug!("Generated JPEG: {}x{}, {} bytes", width, height, jpeg_data.len());
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

    // ... 保持之前的 draw_display_info, draw_text, draw_character 方法不变
    fn draw_display_info(&self, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, display_index: u8, color: [u8; 3]) {
        // 实现保持不变
        let border_width = 10;
        let (width, height) = (img.width(), img.height());

        for x in 0..width {
            for y in 0..border_width {
                img.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
                img.put_pixel(x, height - 1 - y, Rgba([color[0], color[1], color[2], 255]));
            }
        }

        for y in 0..height {
            for x in 0..border_width {
                img.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
                img.put_pixel(width - 1 - x, y, Rgba([color[0], color[1], color[2], 255]));
            }
        }

        let center_x = width / 2 - 100;
        let center_y = height / 2 - 50;

        self.draw_text(img, center_x, center_y, &format!("DISPLAY {}", display_index + 1), color);
        self.draw_text(img, center_x, center_y + 80, "Rotascope Server", color);
        self.draw_text(img, center_x, center_y + 160, &format!("{}x{}", width, height), color);
    }

    fn draw_text(&self, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32, y: u32, text: &str, color: [u8; 3]) {
        // 实现保持不变
        let font_size = 8;

        for (char_index, ch) in text.chars().enumerate() {
            let char_x = x + (char_index as u32 * font_size * 2);

            if char_x >= img.width() {
                break;
            }

            self.draw_character(img, char_x, y, ch, color, font_size);
        }
    }

    fn draw_character(&self, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, x: u32, y: u32, ch: char, color: [u8; 3], size: u32) {
        // 实现保持不变
        // ... 字符绘制逻辑
    }
}