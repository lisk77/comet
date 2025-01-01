use comet_math::Mat4;

mod camera;
pub mod renderer;
pub mod renderer2d;
mod render_pass;
mod render_group;

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) { self.aspect = width as f32 / height as f32; }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::perspective_matrix(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

