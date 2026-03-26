use ab_glyph::{point, Font as AbFont, FontArc, Glyph, PxScale, ScaleFont};
use comet_log::error;
use image::{DynamicImage, Rgba, RgbaImage};

#[derive(Clone)]
pub struct GlyphData {
    pub name: String,
    pub render: DynamicImage,
    pub advance: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Clone)]
pub struct Font {
    name: String,
    data: Vec<u8>,
}

impl Font {
    pub fn from_raw(data: Vec<u8>, name: String) -> Self {
        Self { name, data }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rasterize(&self, size: f32) -> Option<(Vec<GlyphData>, f32)> {
        let font = match FontArc::try_from_vec(self.data.clone()) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to parse font '{}': {}", self.name, e);
                return None;
            }
        };

        let scale = PxScale::from(size);
        let scaled_font = font.as_scaled(scale);
        let mut glyphs: Vec<GlyphData> = Vec::new();

        for code_point in 0x0020u32..=0x007E {
            let ch = match std::char::from_u32(code_point) {
                Some(c) => c,
                None => continue,
            };
            let glyph_id = font.glyph_id(ch);
            if glyph_id.0 == 0 {
                continue;
            }

            if ch == ' ' {
                glyphs.push(GlyphData {
                    name: ch.to_string(),
                    render: DynamicImage::new_rgba8(0, 0),
                    advance: scaled_font.h_advance(glyph_id),
                    offset_x: 0.0,
                    offset_y: 0.0,
                });
                continue;
            }

            let glyph = Glyph { id: glyph_id, scale, position: point(0.0, 0.0) };
            if let Some(outline) = scaled_font.outline_glyph(glyph) {
                let bounds = outline.px_bounds();
                let width = bounds.width().ceil() as u32;
                let height = bounds.height().ceil() as u32;
                if width == 0 || height == 0 {
                    continue;
                }

                let mut image = RgbaImage::new(width, height);
                for pixel in image.pixels_mut() {
                    *pixel = Rgba([0, 0, 0, 0]);
                }
                outline.draw(|x, y, v| {
                    image.put_pixel(x, y, Rgba([255, 255, 255, (v * 255.0) as u8]));
                });

                glyphs.push(GlyphData {
                    name: ch.to_string(),
                    render: DynamicImage::ImageRgba8(image),
                    advance: scaled_font.h_advance(glyph_id),
                    offset_x: bounds.min.x,
                    offset_y: bounds.min.y,
                });
            }
        }

        Some((glyphs, scaled_font.ascent() - scaled_font.descent()))
    }
}
