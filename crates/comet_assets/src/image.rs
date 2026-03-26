use anyhow::*;
use image::{DynamicImage, RgbaImage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
}

#[derive(Debug)]
pub struct Image {
    data: Vec<u8>,
    width: u32,
    height: u32,
    format: ImageFormat,
}

impl Image {
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: ImageFormat) -> Self {
        Self {
            data,
            width,
            height,
            format,
        }
    }

    pub fn from_bytes(bytes: &[u8], is_normal_map: bool) -> Result<Self> {
        let image = image::load_from_memory(bytes)?;
        Ok(Self::from_dynamic_image(&image, is_normal_map))
    }

    pub fn from_dynamic_image(image: &DynamicImage, is_normal_map: bool) -> Self {
        Self {
            data: image.to_rgba8().into_raw(),
            width: image.width(),
            height: image.height(),
            format: if is_normal_map {
                ImageFormat::Rgba8Unorm
            } else {
                ImageFormat::Rgba8UnormSrgb
            },
        }
    }

    pub fn to_dynamic_image(&self) -> Result<DynamicImage> {
        let rgba = RgbaImage::from_raw(self.width, self.height, self.data.clone())
            .ok_or_else(|| anyhow!("Failed to create image from raw data"))?;
        Ok(DynamicImage::ImageRgba8(rgba))
    }

    pub fn into_dynamic_image(self) -> Result<DynamicImage> {
        let rgba = RgbaImage::from_raw(self.width, self.height, self.data)
            .ok_or_else(|| anyhow!("Failed to create image from raw data"))?;
        Ok(DynamicImage::ImageRgba8(rgba))
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Drops the CPU-side pixel data after it has been uploaded to the GPU.
    /// Width and height are preserved so atlas layout remains valid.
    pub fn evict_pixels(&mut self) {
        self.data = Vec::new();
    }

    pub fn is_evicted(&self) -> bool {
        self.data.is_empty()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> ImageFormat {
        self.format
    }
}
