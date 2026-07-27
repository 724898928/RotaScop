pub mod jpeg;

use anyhow::Result;
use openh264::encoder::{BitRate, Encoder, EncoderConfig, FrameRate};
use openh264::formats::YUVBuffer;
use openh264::OpenH264API;

use crate::capture::{Frame, PixelFormat};

pub struct VideoEncoder {
    encoder: Encoder,
    width: u32,
    height: u32,
}

impl VideoEncoder {
    pub fn new(width: u32, height: u32, quality: u8) -> Result<Self> {
        let config = EncoderConfig::new()
            .bitrate(BitRate::from_bps(width * height * quality as u32 * 2))
            .max_frame_rate(FrameRate::from_hz(60.0));
        let api = OpenH264API::from_source();
        let encoder = Encoder::with_api_config(api, config)?;
        Ok(Self {
            encoder,
            width,
            height,
        })
    }

    pub fn encode_frame(&mut self, frame: &Frame) -> Result<Vec<u8>> {
        match frame.format {
            PixelFormat::Bgra => self.encode_bgra(&frame.data),
            PixelFormat::Rgba => self.encode_rgba(&frame.data),
            PixelFormat::Nv12 => self.encode_nv12(&frame.data),
        }
    }

    fn encode_bgra(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let yuv = self.bgra_to_yuv420(data);
        let stream = self.encoder.encode(&yuv)?;
        Ok(stream.to_vec())
    }

    fn encode_rgba(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let yuv = self.rgba_to_yuv420(data);
        let stream = self.encoder.encode(&yuv)?;
        Ok(stream.to_vec())
    }

    fn encode_nv12(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let width = self.width as usize;
        let height = self.height as usize;
        let yuv = YUVBuffer::from_vec(data.to_vec(), width, height);
        let stream = self.encoder.encode(&yuv)?;
        Ok(stream.to_vec())
    }

    fn bgra_to_yuv420(&self, bgra: &[u8]) -> YUVBuffer {
        let width = self.width as usize;
        let height = self.height as usize;
        let mut y_data = vec![0u8; width * height];
        let mut u_data = vec![0u8; width * height / 4];
        let mut v_data = vec![0u8; width * height / 4];

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let b = bgra[idx] as f32;
                let g = bgra[idx + 1] as f32;
                let r = bgra[idx + 2] as f32;

                let y_val = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
                y_data[y * width + x] = y_val;

                if y % 2 == 0 && x % 2 == 0 {
                    let uv_idx = (y / 2) * (width / 2) + (x / 2);
                    u_data[uv_idx] =
                        ((-0.169 * r - 0.331 * g + 0.5 * b + 128.0) as i32).clamp(0, 255) as u8;
                    v_data[uv_idx] =
                        ((0.5 * r - 0.419 * g - 0.081 * b + 128.0) as i32).clamp(0, 255) as u8;
                }
            }
        }

        y_data.extend(u_data);
        y_data.extend(v_data);
        YUVBuffer::from_vec(y_data, width, height)
    }

    fn rgba_to_yuv420(&self, rgba: &[u8]) -> YUVBuffer {
        let width = self.width as usize;
        let height = self.height as usize;
        let mut y_data = vec![0u8; width * height];
        let mut u_data = vec![0u8; width * height / 4];
        let mut v_data = vec![0u8; width * height / 4];

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let r = rgba[idx] as f32;
                let g = rgba[idx + 1] as f32;
                let b = rgba[idx + 2] as f32;

                let y_val = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
                y_data[y * width + x] = y_val;

                if y % 2 == 0 && x % 2 == 0 {
                    let uv_idx = (y / 2) * (width / 2) + (x / 2);
                    u_data[uv_idx] =
                        ((-0.169 * r - 0.331 * g + 0.5 * b + 128.0) as i32).clamp(0, 255) as u8;
                    v_data[uv_idx] =
                        ((0.5 * r - 0.419 * g - 0.081 * b + 128.0) as i32).clamp(0, 255) as u8;
                }
            }
        }

        y_data.extend(u_data);
        y_data.extend(v_data);
        YUVBuffer::from_vec(y_data, width, height)
    }
}
