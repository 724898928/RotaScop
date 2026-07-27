use anyhow::Result;
use image::codecs::jpeg::JpegEncoder as ImgJpegEncoder;
use image::RgbaImage;

use crate::capture::{Frame, PixelFormat};

#[allow(dead_code)]
pub struct JpegEncoder {
    quality: u8,
}

impl JpegEncoder {
    pub fn new(quality: u8) -> Self {
        Self { quality }
    }

    pub fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>> {
        let width = frame.width as usize;
        let height = frame.height as usize;
        let rgba = match frame.format {
            PixelFormat::Bgra => self.bgra_to_rgba(&frame.data, width, height),
            PixelFormat::Rgba => frame.data.clone(),
            PixelFormat::Nv12 => self.nv12_to_rgba(&frame.data, width, height),
        };

        let image = RgbaImage::from_raw(frame.width, frame.height, rgba)
            .ok_or_else(|| anyhow::anyhow!("failed to create RGBA image"))?;

        let mut jpeg_bytes = Vec::with_capacity(200_000);
        let mut encoder = ImgJpegEncoder::new_with_quality(&mut jpeg_bytes, self.quality);
        encoder.encode_image(&image)?;

        Ok(jpeg_bytes)
    }

    fn bgra_to_rgba(&self, bgra: &[u8], width: usize, height: usize) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(width * height * 4);
        for chunk in bgra.chunks(4) {
            if chunk.len() >= 4 {
                rgba.push(chunk[2]);
                rgba.push(chunk[1]);
                rgba.push(chunk[0]);
                rgba.push(255);
            }
        }
        rgba
    }

    fn nv12_to_rgba(&self, nv12: &[u8], width: usize, height: usize) -> Vec<u8> {
        let mut rgba = vec![0u8; width * height * 4];
        let y_size = width * height;
        let y_plane = &nv12[..y_size.min(nv12.len())];
        let uv_plane = &nv12[y_size..];

        for y in 0..height {
            for x in 0..width {
                let y_idx = y * width + x;
                let y_val = *y_plane.get(y_idx).unwrap_or(&128) as f32;

                let uv_x = x / 2;
                let uv_y = y / 2;
                let uv_idx = uv_y * (width / 2) + uv_x;
                let u_val = uv_plane.get(uv_idx * 2).copied().unwrap_or(128) as f32 - 128.0;
                let v_val = uv_plane.get(uv_idx * 2 + 1).copied().unwrap_or(128) as f32 - 128.0;

                let r = (y_val + 1.402 * v_val).clamp(0.0, 255.0) as u8;
                let g = (y_val - 0.344 * u_val - 0.714 * v_val).clamp(0.0, 255.0) as u8;
                let b = (y_val + 1.772 * u_val).clamp(0.0, 255.0) as u8;

                let dst = y_idx * 4;
                rgba[dst] = r;
                rgba[dst + 1] = g;
                rgba[dst + 2] = b;
                rgba[dst + 3] = 255;
            }
        }

        rgba
    }
}
