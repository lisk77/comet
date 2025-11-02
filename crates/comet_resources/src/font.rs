use crate::texture_atlas::{TextureAtlas, TextureRegion};
use ab_glyph::{point, Font as AbFont, FontArc, Glyph, PxScale, ScaleFont};
use image::{DynamicImage, Rgba, RgbaImage};

pub struct GlyphData {
    pub name: String,
    pub render: DynamicImage,
    pub advance: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

pub struct Font {
    name: String,
    size: f32,
    line_height: f32,
    glyphs: TextureAtlas,
}

impl Font {
    pub fn new(path: &str, size: f32) -> Self {
        let (glyphs, line_height) = Self::generate_atlas(path, size);
        Font {
            name: path.to_string(),
            size,
            line_height,
            glyphs,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn size(&self) -> f32 {
        self.size
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    pub fn glyphs(&self) -> &TextureAtlas {
        &self.glyphs
    }

    pub fn get_glyph(&self, ch: char) -> Option<&TextureRegion> {
        self.glyphs.textures().get(&ch.to_string())
    }

    fn generate_atlas(path: &str, size: f32) -> (TextureAtlas, f32) {
        let font_data = std::fs::read(path).expect("Failed to read font file");
        let font = FontArc::try_from_vec(font_data).expect("Failed to load font");

        let scale = PxScale::from(size);
        let scaled_font = font.as_scaled(scale);

        let mut glyphs: Vec<GlyphData> = Vec::new();

        for code_point in 0x0020..=0x007E {
            if let Some(ch) = std::char::from_u32(code_point) {
                let glyph_id = font.glyph_id(ch);
                if glyph_id.0 == 0 {
                    continue;
                }

                if ch == ' ' {
                    let advance = scaled_font.h_advance(glyph_id);
                    glyphs.push(GlyphData {
                        name: ch.to_string(),
                        render: DynamicImage::new_rgba8(0, 0), // no bitmap
                        advance,
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

                if let Some(outline) = scaled_font.outline_glyph(glyph.clone()) {
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
                        let alpha = (v * 255.0) as u8;
                        image.put_pixel(x, y, Rgba([255, 255, 255, alpha]));
                    });

                    glyphs.push(GlyphData {
                        name: ch.to_string(),
                        render: DynamicImage::ImageRgba8(image),
                        advance: scaled_font.h_advance(glyph_id),
                        offset_x: bounds.min.x,
                        offset_y: bounds.min.y,
                    })
                }
            }
        }

        (
            TextureAtlas::from_glyphs(glyphs),
            scaled_font.ascent() - scaled_font.descent(),
        )
    }
}
