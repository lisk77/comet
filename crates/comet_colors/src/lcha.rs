use crate::{sRgba, Color, Hsla, Hsva, Hwba, Laba, LinearRgba, Oklaba, Oklcha, Xyza};
use comet_math::{v4, Deg, Rad};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Lcha {
    lightness: f32,
    chroma: f32,
    hue: Deg,
    alpha: f32,
}

impl Lcha {
    pub fn new(lightness: f32, chroma: f32, hue: impl Into<Deg>, alpha: f32) -> Self {
        let hue = hue.into();
        assert!((0.0..=1.5).contains(&lightness) && (0.0..=1.5).contains(&chroma) && (0.0..=360.0).contains(&hue.degrees()) && (0.0..=1.0).contains(&alpha), "Ligthness needs to be in range 0..1.5\nChroma needs to be in range 0..1.5\nHue needs to be in range 0..360\nAlpha needs to be in range 0..1");
        Self {
            lightness,
            chroma,
            hue,
            alpha,
        }
    }

    pub fn lightness(&self) -> f32 {
        self.lightness
    }

    pub fn chroma(&self) -> f32 {
        self.chroma
    }

    pub fn hue(&self) -> Deg {
        self.hue
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn from_laba(laba: Laba) -> Self {
        let hue = Deg::from(Rad::new(laba.b().atan2(laba.a()))).degrees();
        Self {
            lightness: laba.lightness(),
            chroma: (laba.a() * laba.a() + laba.b() * laba.b()).sqrt(),
            hue: Deg::new(if hue >= 0.0 { hue } else { hue + 360.0 }),
            alpha: laba.alpha(),
        }
    }

    pub fn to_laba(&self) -> Laba {
        Laba::new(
            self.lightness,
            self.chroma * self.hue.to_radians().radians().cos(),
            self.chroma * self.hue.to_radians().radians().sin(),
            self.alpha,
        )
    }

    pub fn to_xyza(&self) -> Xyza {
        self.to_laba().to_xyza()
    }

    pub fn to_linear(&self) -> LinearRgba {
        self.to_xyza().to_linear()
    }

    pub fn to_oklaba(&self) -> Oklaba {
        self.to_linear().to_oklaba()
    }

    pub fn to_oklcha(&self) -> Oklcha {
        self.to_oklaba().to_oklcha()
    }

    pub fn to_rgba(&self) -> sRgba<f32> {
        self.to_linear().to_rgba()
    }

    pub fn to_rgba8(&self) -> sRgba<u8> {
        self.to_linear().to_rgba8()
    }

    pub fn to_hwba(&self) -> Hwba {
        self.to_rgba().to_hwba()
    }

    pub fn to_hsva(&self) -> Hsva {
        self.to_hwba().to_hsva()
    }

    pub fn to_hsla(&self) -> Hsla {
        self.to_hsva().to_hsla()
    }
}

impl Color for Lcha {
    fn to_wgpu(&self) -> wgpu::Color {
        self.to_linear().to_wgpu()
    }
    fn to_linear(&self) -> LinearRgba {
        self.to_linear()
    }

    fn to_vec(&self) -> v4 {
        v4::new(self.lightness, self.chroma, self.hue.degrees(), self.alpha)
    }

    fn from_vec(color: v4) -> Self {
        Self::new(color.x(), color.y(), Deg::new(color.z()), color.w())
    }
}
