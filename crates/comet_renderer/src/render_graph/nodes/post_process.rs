use std::any::Any;
use std::sync::Arc;
use crate::{
    gpu_texture::GpuTexture,
    render_pass::LoadOp,
};
use super::super::node::{BuildContext, NodeState, RenderNode};

pub struct PostProcessNode {
    label: String,
    declared_inputs: Vec<String>,
    declared_output: Option<String>,
    declared_render_target: Option<String>,
    declared_output_format: Option<wgpu::TextureFormat>,
    declared_load: LoadOp,
    shader_src: String,
    pipeline: Option<Arc<wgpu::RenderPipeline>>,
    input_layouts: Vec<Arc<wgpu::BindGroupLayout>>,
    sampler: Option<Arc<wgpu::Sampler>>,
    cached_bind_groups: Option<Vec<Arc<wgpu::BindGroup>>>,
    cached_input_ptrs: Vec<*const wgpu::Texture>,
}

unsafe impl Send for PostProcessNode {}
unsafe impl Sync for PostProcessNode {}

impl PostProcessNode {
    pub fn new(
        label: String,
        inputs: Vec<String>,
        output: Option<String>,
        render_target: Option<String>,
        output_format: Option<wgpu::TextureFormat>,
        load: LoadOp,
        shader_src: String,
    ) -> Self {
        Self {
            label,
            declared_inputs: inputs,
            declared_output: output,
            declared_render_target: render_target,
            declared_output_format: output_format,
            declared_load: load,
            shader_src,
            pipeline: None,
            input_layouts: Vec::new(),
            sampler: None,
            cached_bind_groups: None,
            cached_input_ptrs: Vec::new(),
        }
    }

    pub fn set_output(&mut self, output: Option<String>) {
        self.declared_output = output;
        self.cached_bind_groups = None;
    }

    pub fn set_render_target(&mut self, render_target: Option<String>) {
        self.declared_render_target = render_target;
    }

    fn rebuild_bind_groups(
        &mut self,
        inputs: &[Arc<GpuTexture>],
        device: &wgpu::Device,
    ) {
        let Some(sampler) = &self.sampler else { return };
        let groups: Vec<Arc<wgpu::BindGroup>> = inputs
            .iter()
            .zip(self.input_layouts.iter())
            .map(|(tex, layout)| {
                Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &tex.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                    label: Some(&format!("{} Input Bind Group", self.label)),
                }))
            })
            .collect();
        self.cached_input_ptrs = inputs
            .iter()
            .map(|t| &t.texture as *const wgpu::Texture)
            .collect();
        self.cached_bind_groups = Some(groups);
    }

    fn inputs_changed(&self, inputs: &[Arc<GpuTexture>]) -> bool {
        if inputs.len() != self.cached_input_ptrs.len() {
            return true;
        }
        inputs
            .iter()
            .zip(self.cached_input_ptrs.iter())
            .any(|(t, &ptr)| &t.texture as *const wgpu::Texture != ptr)
    }
}

impl RenderNode for PostProcessNode {
    fn name(&self) -> &str { &self.label }

    fn inputs(&self) -> Vec<&str> {
        self.declared_inputs.iter().map(|s| s.as_str()).collect()
    }

    fn output(&self) -> Option<&str> {
        self.declared_output.as_deref()
    }

    fn render_target(&self) -> Option<&str> {
        self.declared_render_target.as_deref()
    }

    fn output_format(&self) -> Option<wgpu::TextureFormat> {
        self.declared_output_format
    }

    fn load_op(&self) -> LoadOp {
        self.declared_load.clone()
    }

    fn build(&mut self, ctx: BuildContext<'_>) {
        let device = ctx.device;
        let input_count = self.declared_inputs.len();

        let input_layouts: Vec<Arc<wgpu::BindGroupLayout>> = (0..input_count)
            .map(|i| {
                Arc::new(device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some(&format!(
                            "{} Input {} Layout",
                            self.label, i
                        )),
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2,
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(
                                    wgpu::SamplerBindingType::Filtering,
                                ),
                                count: None,
                            },
                        ],
                    },
                ))
            })
            .collect();

        let sampler = Arc::new(device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));

        let layout_refs: Vec<&wgpu::BindGroupLayout> =
            input_layouts.iter().map(|l| l.as_ref()).collect();

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Pipeline Layout", self.label)),
                bind_group_layouts: &layout_refs,
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", self.label)),
            source: wgpu::ShaderSource::Wgsl(self.shader_src.clone().into()),
        });

        let output_format =
            self.declared_output_format.unwrap_or(ctx.format);

        let pipeline = Arc::new(device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{} Pipeline", self.label)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: output_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            },
        ));

        self.input_layouts = input_layouts;
        self.sampler = Some(sampler);
        self.pipeline = Some(pipeline);
    }

    fn run<'rpass>(
        &mut self,
        rpass: &mut wgpu::RenderPass<'rpass>,
        state: &NodeState<'_>,
    ) {
        if self.inputs_changed(state.inputs) || self.cached_bind_groups.is_none() {
            self.rebuild_bind_groups(state.inputs, state.device);
        }

        let Some(pipeline) = &self.pipeline else { return };
        let Some(groups) = &self.cached_bind_groups else { return };

        rpass.set_pipeline(pipeline);
        for (i, group) in groups.iter().enumerate() {
            rpass.set_bind_group(i as u32, group.as_ref(), &[]);
        }
        rpass.draw(0..3, 0..1);
    }

    fn on_resize(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _width: u32,
        _height: u32,
    ) {
        self.cached_bind_groups = None;
        self.cached_input_ptrs.clear();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
