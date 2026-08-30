use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Anchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct ScreenPosition {
    anchor: Anchor,
    offset: v2,
}

impl ScreenPosition {
    pub fn new(anchor: Anchor) -> Self {
        Self {
            anchor,
            offset: v2::ZERO,
        }
    }

    pub fn with_offset(mut self, offset: v2) -> Self {
        self.offset = offset;
        self
    }

    pub fn anchor(&self) -> Anchor {
        self.anchor
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.anchor = anchor;
    }

    pub fn offset(&self) -> v2 {
        self.offset
    }

    pub fn set_offset(&mut self, offset: v2) {
        self.offset = offset;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResolutionScaling {
    /// Fits the virtual canvas height to the output and lets visible width follow its aspect ratio.
    FitVertical,
    /// Fits the virtual canvas width to the output and lets visible height follow its aspect ratio.
    FitHorizontal,
    /// Shows the entire virtual canvas, adding letterboxing or pillarboxing when necessary.
    Fit,
    /// Fills the output while cropping virtual canvas content on one axis when necessary.
    Fill,
    /// Fills the output by revealing additional world beyond the virtual canvas on one axis.
    #[default]
    Expand,
    /// Maps the complete virtual canvas to the output without preserving its aspect ratio.
    Stretch,
}

/// A camera's output rectangle in physical render-target pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraViewport {
    x: Px,
    y: Px,
    width: Px,
    height: Px,
}

impl CameraViewport {
    pub fn new(x: Px, y: Px, width: Px, height: Px) -> Self {
        assert!(
            x.pixels().is_finite()
                && y.pixels().is_finite()
                && width.pixels().is_finite()
                && height.pixels().is_finite()
                && x.pixels() >= 0.0
                && y.pixels() >= 0.0
                && width.pixels() > 0.0
                && height.pixels() > 0.0,
            "camera viewport must have a finite, non-negative origin and positive dimensions"
        );
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(&self) -> Px {
        self.x
    }
    pub fn y(&self) -> Px {
        self.y
    }
    pub fn width(&self) -> Px {
        self.width
    }
    pub fn height(&self) -> Px {
        self.height
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
    virtual_resolution: Option<ScreenSize>,
    resolution_scaling: ResolutionScaling,
    viewport: Option<CameraViewport>,
}

impl Component for Screen {}

impl Default for Screen {
    fn default() -> Self {
        Self {
            virtual_resolution: None,
            resolution_scaling: ResolutionScaling::default(),
            viewport: None,
        }
    }
}

impl Screen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_virtual_resolution(
        mut self,
        width: impl Into<ScreenUnit>,
        height: impl Into<ScreenUnit>,
    ) -> Self {
        self.set_virtual_resolution(width, height);
        self
    }

    pub fn with_resolution_scaling(mut self, scaling: ResolutionScaling) -> Self {
        self.resolution_scaling = scaling;
        self
    }

    pub fn with_viewport(mut self, viewport: CameraViewport) -> Self {
        self.viewport = Some(viewport);
        self
    }

    pub fn virtual_resolution(&self) -> Option<ScreenSize> {
        self.virtual_resolution
    }

    pub fn set_virtual_resolution(
        &mut self,
        width: impl Into<ScreenUnit>,
        height: impl Into<ScreenUnit>,
    ) {
        let resolution = ScreenSize::new(width, height);
        let resolved = resolution.resolve(1.0);
        assert!(
            resolved.x().is_finite()
                && resolved.y().is_finite()
                && resolved.x() > 0.0
                && resolved.y() > 0.0,
            "virtual resolution dimensions must be finite and greater than zero"
        );
        self.virtual_resolution = Some(resolution);
    }

    pub fn clear_virtual_resolution(&mut self) {
        self.virtual_resolution = None;
    }

    pub fn resolution_scaling(&self) -> ResolutionScaling {
        self.resolution_scaling
    }

    pub fn set_resolution_scaling(&mut self, scaling: ResolutionScaling) {
        self.resolution_scaling = scaling;
    }

    pub fn viewport(&self) -> Option<CameraViewport> {
        self.viewport
    }

    pub fn set_viewport(&mut self, viewport: CameraViewport) {
        self.viewport = Some(viewport);
    }

    pub fn clear_viewport(&mut self) {
        self.viewport = None;
    }
}
