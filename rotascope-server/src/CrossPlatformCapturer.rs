use image::{ExtendedColorType, ImageBuffer, Rgba};
use rotascope_core::Result;
use scrap::{Capturer, Display};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::io::ErrorKind::WouldBlock;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;

pub struct CrossPlatformCapturer {
    capturer: Capturer,
    pub width: usize,
    pub height: usize,
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

    pub fn capture_frame(&mut self) -> Result<Vec<u8>> {
        use image::RgbaImage;
        use std::io::ErrorKind::WouldBlock;
        loop {
            match self.capturer.frame() {
                Ok(buffer) => {
                    // let expected = self.width * self.height * 4;
                    // if buffer.len() != expected {
                    //     return Err(format!(
                    //     "Invalid buffer length {}, expected {}",
                    //     buffer.len(),
                    //     expected
                    // ));
                    // }
                    //
                    // let mut rgba = Vec::with_capacity(expected);
                    //
                    // // ---- 高速 BGRA → RGBA 转换（无 bounds check）----
                    // for chunk in buffer.chunks_exact(4) {
                    //     unsafe {
                    //         // chunk: [B, G, R, X]
                    //         let b = *chunk.get_unchecked(0);
                    //         let g = *chunk.get_unchecked(1);
                    //         let r = *chunk.get_unchecked(2);
                    //
                    //         rgba.extend_from_slice(&[r, g, b, 255]);
                    //     }
                    // }
                    //
                    // let img = RgbaImage::from_raw(self.width as u32, self.height as u32, rgba)
                    //     .ok_or_else(||"Failed to create image buffer".to_string())?;
                    return Ok(buffer.to_vec());
                }

                Err(ref e) if e.kind() == WouldBlock => {
                    std::thread::sleep(Duration::from_micros(500)); // 更短延迟
                }

                Err(e) => return Err(e.to_string()),
            }
        }
    }


}
pub fn compress_frame(frame: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Vec<u8>> {
    use image::ExtendedColorType;
    use image::codecs::jpeg::JpegEncoder;

    let w = frame.width();
    let h = frame.height();
    let buffer = frame.as_raw(); // RGBA slice

    let mut rgb = Vec::with_capacity((w * h * 3) as usize);

    // ---- 高速 RGBA → RGB ----
    for px in buffer.chunks_exact(4) {
        rgb.extend_from_slice(&px[..3]); // 直接 [R,G,B]
    }

    let mut out = Vec::new();
    let mut encoder = JpegEncoder::new_with_quality(&mut out, 70);

    encoder
        .encode(&rgb, w, h, ExtendedColorType::Rgb8)
        .map_err(|e| e.to_string())?;

    Ok(out)
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
