use crate::msdf_coloring::edge_coloring_ink_trap;
use crate::AssetSettings;
use ab_glyph::{point, Font as AbFont, FontArc, Glyph, PxScale, ScaleFont};
use comet_log::error;
use comet_math::Px;
use fdsm::{
    bezier::scanline::FillRule,
    correct_error::{correct_error_mtsdf, ErrorCorrectionConfig},
    generate::generate_mtsdf,
    render::correct_sign_mtsdf,
    transform::Transform,
};
use fdsm_image::Rgba32FImage as FdsmRgbaImage;
use fdsm_ttf_parser::{load_shape_from_face, ttf_parser::Face};
use image::{DynamicImage, Rgba, RgbaImage};
use nalgebra::{Affine2, Matrix3};

#[derive(Clone)]
pub struct GlyphData {
    pub name: String,
    pub render: DynamicImage,
    pub advance: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontRasterization {
    #[default]
    Auto,
    Bitmap,
    Pixel,
    Mtsdf,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontSettings {
    rasterization: FontRasterization,
    mtsdf_generation_size: Px,
    mtsdf_range: f64,
}

impl FontSettings {
    const DEFAULT_MTSDF_GENERATION_SIZE: Px = Px::new(64.0);
    const DEFAULT_MTSDF_RANGE: f64 = 4.0;

    pub const fn new() -> Self {
        Self::auto()
    }

    pub const fn auto() -> Self {
        Self::with_rasterization(FontRasterization::Auto)
    }

    pub const fn bitmap() -> Self {
        Self::with_rasterization(FontRasterization::Bitmap)
    }

    pub const fn pixel() -> Self {
        Self::with_rasterization(FontRasterization::Pixel)
    }

    pub const fn mtsdf() -> Self {
        Self::with_rasterization(FontRasterization::Mtsdf)
    }

    const fn with_rasterization(rasterization: FontRasterization) -> Self {
        Self {
            rasterization,
            mtsdf_generation_size: Self::DEFAULT_MTSDF_GENERATION_SIZE,
            mtsdf_range: Self::DEFAULT_MTSDF_RANGE,
        }
    }

    pub fn with_mtsdf_generation(mut self, generation_size: Px, range: f64) -> Self {
        self.mtsdf_generation_size = generation_size;
        self.mtsdf_range = range;
        self
    }

    pub const fn rasterization(self) -> FontRasterization {
        self.rasterization
    }

    pub const fn mtsdf_generation_size(self) -> Px {
        self.mtsdf_generation_size
    }

    pub const fn mtsdf_range(self) -> f64 {
        self.mtsdf_range
    }
}

impl Default for FontSettings {
    fn default() -> Self {
        Self::auto()
    }
}

impl AssetSettings for FontSettings {
    type Asset = Font;

    fn load(&self, bytes: &[u8], path: &str) -> anyhow::Result<Font> {
        Ok(Font {
            name: path.to_string(),
            data: bytes.to_vec(),
            settings: *self,
        })
    }
}

#[derive(Clone)]
pub struct Font {
    name: String,
    data: Vec<u8>,
    settings: FontSettings,
}

impl Font {
    pub fn from_raw(data: Vec<u8>, name: String) -> Self {
        Self {
            name,
            data,
            settings: FontSettings::auto(),
        }
    }

    pub const fn settings(&self) -> FontSettings {
        self.settings
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rasterize(&self, size: Px) -> Option<(Vec<GlyphData>, f32)> {
        let font = match FontArc::try_from_vec(self.data.clone()) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to parse font '{}': {}", self.name, e);
                return None;
            }
        };

        let scale = PxScale::from(size.pixels());
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

            let glyph = Glyph {
                id: glyph_id,
                scale,
                position: point(0.0, 0.0),
            };
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

    /// Generates MTSDF bitmaps for all supported printable ASCII glyphs.
    /// RGB contains the multi-channel distance field and alpha contains the true SDF.
    pub fn rasterize_mtsdf(
        &self,
        generation_size: Px,
        range: f64,
    ) -> Option<(Vec<GlyphData>, f32)> {
        if generation_size.pixels() <= 0.0 || !range.is_finite() || range <= 0.0 {
            return None;
        }

        let face = match Face::parse(&self.data, 0) {
            Ok(face) => face,
            Err(e) => {
                error!("Failed to parse font '{}': {}", self.name, e);
                return None;
            }
        };
        let scale = generation_size.pixels() as f64 / face.units_per_em() as f64;
        let line_height = (face.ascender() as f64 - face.descender() as f64) * scale;
        let mut glyphs = Vec::new();

        for ch in ' '..='~' {
            let Some(glyph_id) = face.glyph_index(ch) else {
                continue;
            };
            let advance = face.glyph_hor_advance(glyph_id).unwrap_or(0) as f64 * scale;
            let Some(bounds) = face.glyph_bounding_box(glyph_id) else {
                glyphs.push(GlyphData {
                    name: ch.to_string(),
                    render: DynamicImage::new_rgba8(0, 0),
                    advance: advance as f32,
                    offset_x: 0.0,
                    offset_y: 0.0,
                });
                continue;
            };

            let glyph_width = (bounds.x_max as f64 - bounds.x_min as f64) * scale;
            let glyph_height = (bounds.y_max as f64 - bounds.y_min as f64) * scale;
            let width = (glyph_width + 2.0 * range).ceil().max(1.0) as u32;
            let height = (glyph_height + 2.0 * range).ceil().max(1.0) as u32;
            let Some(mut shape) = load_shape_from_face(&face, glyph_id) else {
                continue;
            };
            let transformation = Affine2::from_matrix_unchecked(Matrix3::new(
                scale,
                0.0,
                range - bounds.x_min as f64 * scale,
                0.0,
                -scale,
                range + bounds.y_max as f64 * scale,
                0.0,
                0.0,
                1.0,
            ));
            shape.transform(&transformation);
            let colored = edge_coloring_ink_trap(shape, 0.03, glyph_id.0 as u64);
            let prepared = colored.prepare();
            let mut mtsdf = FdsmRgbaImage::new(width, height);
            generate_mtsdf(&prepared, range, &mut mtsdf);
            correct_error_mtsdf(
                &mut mtsdf,
                &colored,
                &prepared,
                range,
                &ErrorCorrectionConfig::default(),
            );
            correct_sign_mtsdf(&mut mtsdf, &prepared, FillRule::Nonzero);
            let pixels = mtsdf
                .into_raw()
                .into_iter()
                .map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
                .collect();
            let image = RgbaImage::from_raw(width, height, pixels)?;

            glyphs.push(GlyphData {
                name: ch.to_string(),
                render: DynamicImage::ImageRgba8(image),
                advance: advance as f32,
                offset_x: bounds.x_min as f32 * scale as f32 - range as f32,
                offset_y: -(bounds.y_max as f32) * scale as f32 - range as f32,
            });
        }

        Some((glyphs, line_height as f32))
    }
}
