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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EulerAngles {
    radians: crate::v3,
}

impl EulerAngles {
    pub const ZERO: Self = Self {
        radians: crate::v3::ZERO,
    };

    pub fn from_degrees(x: f32, y: f32, z: f32) -> Self {
        Self {
            radians: crate::v3::new(x.to_radians(), y.to_radians(), z.to_radians()),
        }
    }

    pub const fn from_radians(x: f32, y: f32, z: f32) -> Self {
        Self {
            radians: crate::v3::new(x, y, z),
        }
    }

    pub fn as_degrees(self) -> crate::v3 {
        crate::v3::new(
            self.radians.x().to_degrees(),
            self.radians.y().to_degrees(),
            self.radians.z().to_degrees(),
        )
    }

    pub const fn as_radians(self) -> crate::v3 {
        self.radians
    }

    pub fn set_x(&mut self, angle: impl Into<Rad>) {
        self.radians.x = angle.into().radians();
    }

    pub fn set_y(&mut self, angle: impl Into<Rad>) {
        self.radians.y = angle.into().radians();
    }

    pub fn set_z(&mut self, angle: impl Into<Rad>) {
        self.radians.z = angle.into().radians();
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
