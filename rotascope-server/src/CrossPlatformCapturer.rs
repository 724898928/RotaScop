use std::fmt;
use std::fmt::{Debug, Formatter};
use scrap::{Capturer, Display};
use image::{ExtendedColorType, ImageBuffer, Rgba};
use std::io::ErrorKind::WouldBlock;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;
use rotascope_core::Result;

pub struct CrossPlatformCapturer {
    capturer: Capturer,
    width: usize,
    height: usize,
}

impl Debug for CrossPlatformCapturer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("CrossPlatformCapturer")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl CrossPlatformCapturer {
    pub fn new_primary() -> Result<Self> {
        let display = Display::primary().map_err(|e| e.to_string())?;
        let width = display.width();
        let height = display.height();

        let capturer = Capturer::new(display).map_err(|e| e.to_string())?;

        Ok(Self {
            capturer,
            width,
            height,
        })
    }

    pub fn capture_frame(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        loop {
            match self.capturer.frame() {
                Ok(buffer) => {
                    let mut image_data = Vec::with_capacity(self.width * &self.height * 4);

                    for i in 0..self.width * self.height {
                        let base = i * 4;
                        if base + 3 < buffer.len() {
                            // BGRA → RGBA
                            image_data.push(buffer[base + 2]); // R
                            image_data.push(buffer[base + 1]); // G
                            image_data.push(buffer[base]);     // B
                            image_data.push(255);              // A
                        }
                    }

                    return Ok(
                        ImageBuffer::from_raw(
                            self.width as u32,
                            self.height as u32,
                            image_data,
                        )
                            .ok_or("Failed to create image buffer")?,
                    );
                }

                Err(ref e) if e.kind() == WouldBlock => {
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }

                Err(e) => return Err(e.to_string()),
            }
        }
    }


}
pub fn compress_frame(frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Vec<u8>> {
    use image::codecs::jpeg::JpegEncoder;
    let mut compressed_data = Vec::new();
    // ---- RGBA → RGB ----
    let mut rgb_data = Vec::with_capacity((frame.width() * frame.height() * 3) as usize);
    for pixel in frame.pixels() {
        rgb_data.push(pixel[0]); // R
        rgb_data.push(pixel[1]); // G
        rgb_data.push(pixel[2]); // B
    }
    // ---- JPEG 编码 ----
    let mut encoder = JpegEncoder::new_with_quality(&mut compressed_data, 70);
    encoder
        .encode(
            &rgb_data,
            frame.width(),
            frame.height(),
            ExtendedColorType::Rgb8
        )
        .map_err(|e| e.to_string())?;

    Ok(compressed_data)
}





//
// // 高性能屏幕流服务器
// pub struct ScreenStreamServer {
//     capturer: CrossPlatformCapturer,
//     frame_rate: u32,
// }
//
// impl ScreenStreamServer {
//     pub fn new(frame_rate: u32) -> Result<Self> {
//         Ok(Self {
//             capturer: CrossPlatformCapturer::new_primary()?,
//             frame_rate,
//         })
//     }
//
//     pub async fn start_websocket_stream(&mut self) -> Result<()> {
//         use tokio_tungstenite::tungstenite::Message;
//
//         // 这里连接到你的WebSocket服务器
//         // 假设我们已经有一个WebSocket连接
//
//         let frame_interval = 1000 / self.frame_rate;
//
//         loop {
//             let start = std::time::Instant::now();
//
//             match self.capturer.capture_frame() {
//                 Ok(frame) => {
//                     // 🔥 性能优化：压缩图像数据
//                     let compressed_data = self.compress_frame(&frame)?;
//
//                     // 通过WebSocket发送压缩后的帧数据
//                     // ws_stream.send(Message::Binary(compressed_data)).await?;
//
//                     println!("发送一帧，大小: {} bytes", compressed_data.len());
//                 }
//                 Err(e) => eprintln!("捕获帧失败: {}", e),
//             }
//
//             // 控制帧率
//             let elapsed = start.elapsed();
//             if elapsed < Duration::from_millis(frame_interval as u64) {
//                 tokio::time::sleep(Duration::from_millis(frame_interval as u64) - elapsed).await;
//             }
//         }
//     }
//
//     // 🔥 图像压缩优化
//     fn compress_frame(&self, frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Vec<u8>> {
//         use image::ImageEncoder;
//         use image::codecs::jpeg::JpegEncoder;
//
//         let mut compressed_data = Vec::new();
//
//         // 使用JPEG编码器压缩（调整质量平衡大小和画质）
//         let mut encoder = JpegEncoder::new_with_quality(&mut compressed_data, 70);
//         encoder.encode(&frame, frame.width(), frame.height(), ExtendedColorType::Rgb8).map_err(|e|e.to_string())?;
//
//         Ok(compressed_data)
//     }
//
//     // 🔥 区域捕获优化 - 只捕获变化区域
//     pub fn capture_changed_region(&mut self, previous_frame: &[u8]) -> Result<Option<ImageBuffer<Rgba<u8>, Vec<u8>>>> {
//         let current_frame = self.capturer.capture_frame()?;
//
//         // 简单的变化检测 - 在实际应用中可以使用更复杂的算法
//         if self.has_significant_changes(&current_frame, previous_frame) {
//             Ok(Some(current_frame))
//         } else {
//             Ok(None)
//         }
//     }
//
//     fn has_significant_changes(&self, current: &ImageBuffer<Rgba<u8>, Vec<u8>>, previous: &[u8]) -> bool {
//         // 简化的变化检测逻辑
//         // 在实际应用中可以使用像素差异阈值等更复杂的方法
//         current.as_raw() != previous
//     }
// }