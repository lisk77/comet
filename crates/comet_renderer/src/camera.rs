use comet_ecs::{Camera2D, Transform2D};
use comet_math::{m4, v2, v3};

pub struct CameraManager {
    cameras: Vec<RenderCamera>,
    active_camera: usize,
}

impl CameraManager {
    pub fn new() -> Self {
        Self {
            cameras: Vec::new(),
            active_camera: 0,
        }
    }

    pub fn get_camera(&self) -> &RenderCamera {
        self.cameras.get(self.active_camera).unwrap()
    }

    pub fn update_from_scene(&mut self, scene: &comet_ecs::Scene, camera_entities: Vec<usize>) {
        self.cameras.clear();

        let mut cameras_with_priority: Vec<(RenderCamera, u8)> = Vec::new();

        for entity in camera_entities {
            let camera_component = scene.get_component::<Camera2D>(entity).unwrap();
            let transform_component = scene.get_component::<Transform2D>(entity).unwrap();

            let render_cam = RenderCamera::new(
                camera_component.zoom(),
                camera_component.dimensions(),
                v3::new(
                    transform_component.position().as_vec().x(),
                    transform_component.position().as_vec().y(),
                    0.0,
                ),
            );

            cameras_with_priority.push((render_cam, camera_component.priority()));
        }

        if cameras_with_priority.is_empty() {
            return;
        }

        cameras_with_priority.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        self.cameras = cameras_with_priority.into_iter().map(|(c, _)| c).collect();
        self.active_camera = 0;
    }

    pub fn has_active_camera(&self) -> bool {
        !self.cameras.is_empty()
    }
}

pub struct RenderCamera {
    zoom: f32,
    dimension: v2,
    position: v3,
}

impl RenderCamera {
    pub fn new(zoom: f32, dimension: v2, position: v3) -> Self {
        Self {
            zoom,
            dimension,
            position,
        }
    }

    pub fn build_view_projection_matrix(&self) -> m4 {
        let zoomed_width = self.dimension.x() / self.zoom;
        let zoomed_height = self.dimension.y() / self.zoom;

        m4::OPENGL_CONV
            * m4::orthographic_projection(
                self.position.x() - zoomed_width / 2.0,
                self.position.x() + zoomed_width / 2.0,
                self.position.y() - zoomed_height / 2.0,
                self.position.y() + zoomed_height / 2.0,
                1.0,
                0.0,
            )
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &RenderCamera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
