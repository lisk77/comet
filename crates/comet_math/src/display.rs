#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Px(f32);

impl Px {
    pub const ZERO: Self = Self(0.0);

    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    pub const fn pixels(self) -> f32 {
        self.0
    }

    pub fn to_dp(self, scale_factor: f32) -> Dp {
        assert!(
            scale_factor.is_finite() && scale_factor > 0.0,
            "display scale factor must be finite and greater than zero"
        );
        Dp(self.0 / scale_factor)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Dp(f32);

impl Dp {
    pub const ZERO: Self = Self(0.0);

    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    pub const fn display_points(self) -> f32 {
        self.0
    }

    pub fn to_px(self, scale_factor: f32) -> Px {
        assert!(
            scale_factor.is_finite() && scale_factor > 0.0,
            "display scale factor must be finite and greater than zero"
        );
        Px(self.0 * scale_factor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ScreenUnit {
    Px(Px),
    Dp(Dp),
}

impl ScreenUnit {
    pub fn resolve(self, scale_factor: f32) -> Px {
        match self {
            Self::Px(value) => value,
            Self::Dp(value) => value.to_px(scale_factor),
        }
    }
}

impl From<Px> for ScreenUnit {
    fn from(value: Px) -> Self {
        Self::Px(value)
    }
}

impl From<Dp> for ScreenUnit {
    fn from(value: Dp) -> Self {
        Self::Dp(value)
    }
}

pub const fn px(value: f32) -> Px {
    Px::new(value)
}

pub const fn dp(value: f32) -> Dp {
    Dp::new(value)
}
