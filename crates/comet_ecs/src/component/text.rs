use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextJustification {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct TextLayout {
    anchor: Anchor,
    justification: TextJustification,
}

impl TextLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_justification(mut self, justification: TextJustification) -> Self {
        self.justification = justification;
        self
    }

    pub fn anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    pub fn justification(&self) -> TextJustification {
        self.justification
    }

    pub fn set_justification(&mut self, justification: TextJustification) {
        self.justification = justification;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextSize {
    Screen(ScreenUnit),
    World(f32),
}

impl Default for TextSize {
    fn default() -> Self {
        Self::Screen(dp(16.0).into())
    }
}

impl TextSize {
    fn assert_valid(self) {
        let value = match self {
            Self::Screen(ScreenUnit::Px(size)) => size.pixels(),
            Self::Screen(ScreenUnit::Dp(size)) => size.display_points(),
            Self::World(size) => size,
        };
        assert!(
            value.is_finite() && value > 0.0,
            "text size must be finite and greater than zero"
        );
    }
}

impl From<Px> for TextSize {
    fn from(size: Px) -> Self {
        Self::Screen(size.into())
    }
}

impl From<Dp> for TextSize {
    fn from(size: Dp) -> Self {
        Self::Screen(size.into())
    }
}

impl From<ScreenUnit> for TextSize {
    fn from(size: ScreenUnit) -> Self {
        Self::Screen(size)
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Text {
    content: String,
    font: comet_assets::Asset<comet_assets::Font>,
    font_size: TextSize,
    color: v4,
    is_visible: bool,
    bounds: v2,
}

impl Text {
    pub fn new(content: impl Into<String>, font: comet_assets::Asset<comet_assets::Font>) -> Self {
        Self {
            content: content.into(),
            font,
            font_size: TextSize::default(),
            color: LinearRgba::new(1.0, 1.0, 1.0, 1.0).to_vec(),
            is_visible: true,
            bounds: v2::ZERO,
        }
    }

    pub fn with_font_size(mut self, font_size: impl Into<TextSize>) -> Self {
        self.set_font_size(font_size);
        self
    }

    pub fn with_world_font_size(mut self, font_size: f32) -> Self {
        self.set_world_font_size(font_size);
        self
    }

    pub fn with_color(mut self, color: impl Color) -> Self {
        self.color = color.to_vec();
        self
    }

    pub fn with_visibility(mut self, is_visible: bool) -> Self {
        self.is_visible = is_visible;
        self
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    pub fn font(&self) -> comet_assets::Asset<comet_assets::Font> {
        self.font
    }

    pub fn set_font(&mut self, font: comet_assets::Asset<comet_assets::Font>) {
        self.font = font;
    }

    pub fn font_size(&self) -> TextSize {
        self.font_size
    }

    pub fn set_font_size(&mut self, font_size: impl Into<TextSize>) {
        let font_size = font_size.into();
        font_size.assert_valid();
        self.font_size = font_size;
    }

    pub fn set_world_font_size(&mut self, font_size: f32) {
        assert!(
            font_size.is_finite() && font_size > 0.0,
            "world font size must be finite and greater than zero"
        );
        self.font_size = TextSize::World(font_size);
    }

    pub fn color(&self) -> impl Color {
        LinearRgba::from_vec(self.color)
    }

    pub fn set_color(&mut self, color: impl Color) {
        self.color = color.to_vec();
    }

    pub fn set_visibility(&mut self, visibility: bool) {
        self.is_visible = visibility
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn bounds(&self) -> v2 {
        self.bounds
    }

    pub fn set_bounds(&mut self, bounds: v2) {
        self.bounds = bounds
    }
}
