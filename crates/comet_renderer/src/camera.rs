use comet_ecs::{Camera, ResolutionScaling, Transform};
use comet_math::{m4, v2, v3};

#[allow(unused)]
pub struct CameraManager {
    cameras: Vec<RenderCamera>,
    active_camera: usize,
}

#[allow(unused)]
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

    pub fn update_from_scene(
        &mut self,
        scene: &comet_ecs::Scene,
        camera_entities: Vec<comet_ecs::Entity>,
    ) {
        self.cameras.clear();

        let mut cameras_with_priority: Vec<(RenderCamera, u8)> = Vec::new();

        for entity in camera_entities {
            let camera_component = scene.get_component::<Camera>(entity).unwrap();
            let transform_component = scene.get_component::<Transform>(entity).unwrap();

            let base_size = camera_component
                .virtual_resolution()
                .unwrap_or_else(|| v2::new(1.0, 1.0));
            let visible_size = base_size / camera_component.magnification();
            let render_cam = RenderCamera::new(
                visible_size,
                v3::new(
                    transform_component.position().x(),
                    transform_component.position().y(),
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ResolvedViewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct ResolvedCameraViewport {
    pub visible_world_size: v2,
    pub viewport: ResolvedViewport,
}

pub fn resolve_camera_viewport(
    virtual_resolution: v2,
    scaling: ResolutionScaling,
    magnification: f32,
    output_bounds: ResolvedViewport,
) -> ResolvedCameraViewport {
    let virtual_width = valid_dimension(virtual_resolution.x());
    let virtual_height = valid_dimension(virtual_resolution.y());
    let magnification = if magnification.is_finite() && magnification > 0.0 {
        magnification
    } else {
        1.0
    };
    let baseline_width = virtual_width / magnification;
    let baseline_height = virtual_height / magnification;
    let output_width = output_bounds.width.max(1) as f32;
    let output_height = output_bounds.height.max(1) as f32;
    let horizontal_scale = output_width / baseline_width;
    let vertical_scale = output_height / baseline_height;
    let full_viewport = ResolvedViewport {
        width: output_bounds.width.max(1),
        height: output_bounds.height.max(1),
        ..output_bounds
    };

    let (visible_width, visible_height, viewport) = match scaling {
        ResolutionScaling::FitVertical => (
            baseline_height * output_width / output_height,
            baseline_height,
            full_viewport,
        ),
        ResolutionScaling::FitHorizontal => (
            baseline_width,
            baseline_width * output_height / output_width,
            full_viewport,
        ),
        ResolutionScaling::Fit => {
            let scale = horizontal_scale.min(vertical_scale);
            let width = (baseline_width * scale).round().max(1.0) as u32;
            let height = (baseline_height * scale).round().max(1.0) as u32;
            let width = width.min(full_viewport.width);
            let height = height.min(full_viewport.height);
            (
                baseline_width,
                baseline_height,
                ResolvedViewport {
                    x: output_bounds.x + (full_viewport.width - width) / 2,
                    y: output_bounds.y + (full_viewport.height - height) / 2,
                    width,
                    height,
                },
            )
        }
        ResolutionScaling::Fill => {
            let scale = horizontal_scale.max(vertical_scale);
            (output_width / scale, output_height / scale, full_viewport)
        }
        ResolutionScaling::Expand => {
            let scale = horizontal_scale.min(vertical_scale);
            (output_width / scale, output_height / scale, full_viewport)
        }
        ResolutionScaling::Stretch => (baseline_width, baseline_height, full_viewport),
    };

    ResolvedCameraViewport {
        visible_world_size: v2::new(visible_width, visible_height),
        viewport,
    }
}

fn valid_dimension(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

pub struct RenderCamera {
    visible_world_size: v2,
    position: v3,
}

impl RenderCamera {
    pub fn new(visible_world_size: v2, position: v3) -> Self {
        Self {
            visible_world_size: v2::new(
                valid_dimension(visible_world_size.x()),
                valid_dimension(visible_world_size.y()),
            ),
            position,
        }
    }

    pub fn build_view_projection_matrix(&self) -> m4 {
        let half_width = self.visible_world_size.x() / 2.0;
        let half_height = self.visible_world_size.y() / 2.0;

        m4::OPENGL_CONV
            * m4::orthographic_projection(
                self.position.x() - half_width,
                self.position.x() + half_width,
                self.position.y() - half_height,
                self.position.y() + half_height,
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

    pub fn set_view_proj(&mut self, mat: [[f32; 4]; 4]) {
        self.view_proj = mat;
    }
}
