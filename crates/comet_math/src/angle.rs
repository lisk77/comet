#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Rad(f32);

impl Rad {
    pub const ZERO: Self = Self(0.0);

    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    pub const fn radians(self) -> f32 {
        self.0
    }

    pub fn to_degrees(self) -> Deg {
        Deg(self.0.to_degrees())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Deg(f32);

impl Deg {
    pub const ZERO: Self = Self(0.0);

    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    pub const fn degrees(self) -> f32 {
        self.0
    }

    pub fn to_radians(self) -> Rad {
        Rad(self.0.to_radians())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum AngleUnit {
    Rad(Rad),
    Deg(Deg),
}

impl AngleUnit {
    pub fn radians(self) -> Rad {
        self.into()
    }
}

impl From<Rad> for AngleUnit {
    fn from(angle: Rad) -> Self {
        Self::Rad(angle)
    }
}

impl From<Deg> for AngleUnit {
    fn from(angle: Deg) -> Self {
        Self::Deg(angle)
    }
}

impl From<AngleUnit> for Rad {
    fn from(angle: AngleUnit) -> Self {
        match angle {
            AngleUnit::Rad(angle) => angle,
            AngleUnit::Deg(angle) => angle.into(),
        }
    }
}

impl From<Deg> for Rad {
    fn from(angle: Deg) -> Self {
        angle.to_radians()
    }
}

impl From<Rad> for Deg {
    fn from(angle: Rad) -> Self {
        angle.to_degrees()
    }
}

pub const fn rad(value: f32) -> Rad {
    Rad::new(value)
}

pub const fn deg(value: f32) -> Deg {
    Deg::new(value)
}
