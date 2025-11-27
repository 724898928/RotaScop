use std::io::ErrorKind::WouldBlock;
use std::time::Duration;
use image::{ImageBuffer, Rgba};
use scrap::{Capturer, Display};
use rotascope_core::Result;
pub struct ScreenCapturer {
    capturer: Capturer,
    width: usize,
    height: usize,
}

impl ScreenCapturer {
    pub fn new_primary() -> Result<Self> {
        let display = Display::primary().map_err(|e|e.to_string())?;
        let width = display.width();
        let height = display.height();

        let capturer = Capturer::new(display).map_err(|e|e.to_string())?;

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
                    let mut image_data = Vec::with_capacity(self.width * self.height * 4);

                    // 将BGRA转换为RGBA格式
                    for i in 0..self.width * self.height {
                        let base = i * 4;
                        if base + 2 < buffer.len() {
                            image_data.push(buffer[base + 2]); // R
                            image_data.push(buffer[base + 1]); // G
                            image_data.push(buffer[base]);     // B
                            image_data.push(255);              // A
                        }
                    }

                    return Ok(ImageBuffer::from_raw(
                        self.width as u32,
                        self.height as u32,
                        image_data
                    ).ok_or("Failed to create image buffer")?);
                }
                Err(ref e) if e.kind() == WouldBlock => {
                    // 没有新帧，继续等待
                    std::thread::sleep(Duration::from_millis(1));
                    continue;
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    }
}