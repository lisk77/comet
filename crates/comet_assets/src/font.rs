use crate::AssetSettings;
use ab_glyph::{point, Font as AbFont, FontArc, Glyph, PxScale, ScaleFont};
use comet_log::error;
use comet_math::Px;
use image::{DynamicImage, Rgba, RgbaImage};

#[derive(Clone)]
pub struct GlyphData {
    pub name: String,
    pub render: DynamicImage,
    pub advance: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FontSettings;

impl FontSettings {
    pub fn new() -> Self {
        Self
    }
}

impl AssetSettings for FontSettings {
    type Asset = Font;

    fn load(&self, bytes: &[u8], path: &str) -> anyhow::Result<Font> {
        Ok(Font::from_raw(bytes.to_vec(), path.to_string()))
    }
}

#[derive(Clone)]
pub struct Font {
    name: String,
    data: Vec<u8>,
}

/// Reusable scratch buffers for `Font::squared_edt_1d`, sized to the longest
/// row/column of the glyph being processed.
struct EdtScratch {
    envelope_x: Vec<usize>,
    boundary: Vec<f32>,
    result: Vec<f32>,
}

impl EdtScratch {
    fn new(max_dim: usize) -> Self {
        Self {
            envelope_x: vec![0usize; max_dim],
            boundary: vec![0.0f32; max_dim + 1],
            result: vec![0.0f32; max_dim],
        }
    }
}

impl Font {
    pub fn from_raw(data: Vec<u8>, name: String) -> Self {
        Self { name, data }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn parsed(&self) -> Option<FontArc> {
        match FontArc::try_from_vec(self.data.clone()) {
            Ok(font) => Some(font),
            Err(e) => {
                error!("Failed to parse font '{}': {}", self.name, e);
                None
            }
        }
    }

    /// Returns the font's ascent-to-descent line height at `size`.
    pub fn line_height(&self, size: Px) -> Option<f32> {
        let font = self.parsed()?;
        let scaled_font = font.as_scaled(PxScale::from(size.pixels()));
        Some(scaled_font.ascent() - scaled_font.descent())
    }

    /// Rasterizes one character at `size`. Characters without an outline (such as
    /// spaces) retain their advance and return an empty image.
    pub fn rasterize_char(&self, ch: char, size: Px) -> Option<GlyphData> {
        let font = self.parsed()?;
        let scale = PxScale::from(size.pixels());
        let scaled_font = font.as_scaled(scale);
        let glyph_id = font.glyph_id(ch);
        if glyph_id.0 == 0 {
            return None;
        }

        let advance = scaled_font.h_advance(glyph_id);
        let glyph = Glyph {
            id: glyph_id,
            scale,
            position: point(0.0, 0.0),
        };
        let Some(outline) = scaled_font.outline_glyph(glyph) else {
            return Some(GlyphData {
                name: ch.to_string(),
                render: DynamicImage::new_rgba8(0, 0),
                advance,
                offset_x: 0.0,
                offset_y: 0.0,
            });
        };

        let bounds = outline.px_bounds();
        let width = bounds.width().ceil() as u32;
        let height = bounds.height().ceil() as u32;
        if width == 0 || height == 0 {
            return Some(GlyphData {
                name: ch.to_string(),
                render: DynamicImage::new_rgba8(0, 0),
                advance,
                offset_x: bounds.min.x,
                offset_y: bounds.min.y,
            });
        }

        let mut image = RgbaImage::new(width, height);
        outline.draw(|x, y, coverage| {
            image.put_pixel(
                x,
                y,
                Rgba([255, 255, 255, (coverage * 255.0).round() as u8]),
            );
        });

        Some(GlyphData {
            name: ch.to_string(),
            render: DynamicImage::ImageRgba8(image),
            advance,
            offset_x: bounds.min.x,
            offset_y: bounds.min.y,
        })
    }

    /// Generates a single-channel signed-distance glyph in the alpha channel.
    /// RGB is white, the contour is alpha 128, and `spread` controls both the
    /// distance range and the transparent padding around the bitmap.
    pub fn rasterize_sdf_char(
        &self,
        ch: char,
        generation_size: Px,
        spread: u32,
    ) -> Option<GlyphData> {
        Self::glyph_to_sdf(self.rasterize_char(ch, generation_size)?, spread)
    }

    fn glyph_to_sdf(glyph: GlyphData, spread: u32) -> Option<GlyphData> {
        let Some(source) = glyph.render.as_rgba8() else {
            return Some(glyph);
        };
        if source.width() == 0 || source.height() == 0 {
            return Some(glyph);
        }

        let width = source.width() + spread * 2;
        let height = source.height() + spread * 2;
        let mut coverage = vec![0.0; (width * height) as usize];
        let mut inside = vec![false; (width * height) as usize];
        for y in 0..source.height() {
            for x in 0..source.width() {
                let index = ((y + spread) * width + x + spread) as usize;
                coverage[index] = source.get_pixel(x, y)[3] as f32 / 255.0;
                inside[index] = coverage[index] >= 0.5;
            }
        }

        let max_distance = spread.max(1) as f32;
        let distance_to_inside = Self::distance_transform(&inside, width, height, true);
        let distance_to_outside = Self::distance_transform(&inside, width, height, false);
        let mut image = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let index = (y * width + x) as usize;
                let signed_distance = if inside[index] {
                    distance_to_outside[index] + coverage[index] - 1.0
                } else {
                    -distance_to_inside[index] + coverage[index]
                };
                let alpha = (0.5 + signed_distance / (2.0 * max_distance)).clamp(0.0, 1.0);
                image.put_pixel(x, y, Rgba([255, 255, 255, (alpha * 255.0).round() as u8]));
            }
        }

        Some(GlyphData {
            name: glyph.name,
            render: DynamicImage::ImageRgba8(image),
            advance: glyph.advance,
            offset_x: glyph.offset_x - spread as f32,
            offset_y: glyph.offset_y - spread as f32,
        })
    }

    /// Exact 2D Euclidean distance transform to the nearest pixel equal to `target`, via
    /// two separable 1D passes (Felzenszwalt & Huttenlocher). Unlike a raster-order
    /// chamfer pass, each row/column here is independent, so there is no pixel-to-pixel
    /// dependency chain to stall on.
    fn distance_transform(mask: &[bool], width: u32, height: u32, target: bool) -> Vec<f32> {
        let width = width as usize;
        let height = height as usize;
        const SENTINEL: f32 = 1.0e5;
        let mut squared: Vec<f32> = mask
            .iter()
            .map(|&value| if value == target { 0.0 } else { SENTINEL })
            .collect();

        let max_dim = width.max(height);
        let mut scratch = EdtScratch::new(max_dim);
        let mut line = vec![0.0f32; max_dim];

        for x in 0..width {
            for y in 0..height {
                line[y] = squared[y * width + x];
            }
            Self::squared_edt_1d(&mut line[..height], &mut scratch);
            for y in 0..height {
                squared[y * width + x] = line[y];
            }
        }

        for y in 0..height {
            let row = &mut squared[y * width..(y + 1) * width];
            line[..width].copy_from_slice(row);
            Self::squared_edt_1d(&mut line[..width], &mut scratch);
            row.copy_from_slice(&line[..width]);
        }

        squared.iter().map(|&value| value.sqrt()).collect()
    }

    /// In-place exact 1D squared distance transform of sampled function `f`, using
    /// caller-provided scratch buffers to avoid allocating per row/column.
    fn squared_edt_1d(f: &mut [f32], scratch: &mut EdtScratch) {
        let n = f.len();
        if n <= 1 {
            return;
        }

        let envelope_x = &mut scratch.envelope_x[..n];
        let boundary = &mut scratch.boundary[..n + 1];
        let mut k = 0usize;
        envelope_x[0] = 0;
        boundary[0] = f32::NEG_INFINITY;
        boundary[1] = f32::INFINITY;
        for q in 1..n {
            let mut s;
            loop {
                let vk = envelope_x[k];
                s = ((f[q] + (q * q) as f32) - (f[vk] + (vk * vk) as f32))
                    / (2.0 * (q as f32 - vk as f32));
                if s <= boundary[k] && k > 0 {
                    k -= 1;
                } else {
                    break;
                }
            }
            k += 1;
            envelope_x[k] = q;
            boundary[k] = s;
            boundary[k + 1] = f32::INFINITY;
        }

        let result = &mut scratch.result[..n];
        let mut k = 0usize;
        for q in 0..n {
            while boundary[k + 1] < q as f32 {
                k += 1;
            }
            let vk = envelope_x[k];
            let dist = q as f32 - vk as f32;
            result[q] = dist * dist + f[vk];
        }
        f.copy_from_slice(result);
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

    pub fn rasterize_sdf(&self, size: Px, spread: u32) -> Option<(Vec<GlyphData>, f32)> {
        let (glyphs, line_height) = self.rasterize(size)?;
        let glyphs = glyphs
            .into_iter()
            .filter_map(|glyph| Self::glyph_to_sdf(glyph, spread))
            .collect();
        Some((glyphs, line_height))
    }
}
