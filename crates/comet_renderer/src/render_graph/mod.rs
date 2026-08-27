pub mod node;
pub mod nodes;

pub use node::{BuildContext, NodeState, RenderNode};
pub use nodes::{PassNode, PostProcessNode};

use crate::gpu_texture::GpuTexture;
use crate::render_pass::LoadOp;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

struct CompiledTexture {
    name: String,
    texture: Arc<GpuTexture>,
}

struct CompiledNode {
    node_index: usize,
    inputs: Vec<Arc<GpuTexture>>,
    target: Option<Arc<GpuTexture>>,
    load: LoadOp,
    pass_label: String,
}

pub struct RenderGraph {
    nodes: Vec<Box<dyn RenderNode>>,
    order_edges: Vec<(String, String)>,
    execution_plan: Vec<CompiledNode>,
    compiled_textures: Vec<CompiledTexture>,
    dirty: bool,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            order_edges: Vec::new(),
            execution_plan: Vec::new(),
            compiled_textures: Vec::new(),
            dirty: true,
        }
    }

    pub fn add_node(
        &mut self,
        mut node: impl RenderNode,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) {
        node.build(BuildContext {
            device,
            queue,
            format,
            width,
            height,
        });
        self.nodes.push(Box::new(node));
        self.dirty = true;
    }

    pub fn remove_node(&mut self, name: &str) {
        if let Some(position) = self.nodes.iter().position(|node| node.name() == name) {
            self.nodes.remove(position);
        }
        self.dirty = true;
    }

    pub fn add_order_edge(&mut self, before: &str, after: &str) {
        self.order_edges
            .push((before.to_string(), after.to_string()));
        self.dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn has_node(&self, name: &str) -> bool {
        self.nodes.iter().any(|node| node.name() == name)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn pass_mut(&mut self, name: &str) -> Option<&mut PassNode> {
        self.nodes
            .iter_mut()
            .find(|node| node.name() == name)?
            .pass_mut()
    }

    pub fn post_process_mut(&mut self, name: &str) -> Option<&mut PostProcessNode> {
        self.nodes
            .iter_mut()
            .find(|node| node.name() == name)?
            .post_process_mut()
    }

    pub fn on_resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        self.execution_plan.clear();
        self.compiled_textures.clear();
        self.dirty = true;
        for node in &mut self.nodes {
            node.on_resize(device, queue, width, height);
        }
    }

    fn compile(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) {
        let node_count = self.nodes.len();
        let name_map: HashMap<&str, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.name(), index))
            .collect();
        let output_nodes: HashMap<&str, usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| node.output().map(|output| (output, index)))
            .collect();

        let mut edges = HashSet::new();
        for (index, node) in self.nodes.iter().enumerate() {
            for input in node.inputs() {
                if let Some(&producer) = output_nodes.get(input.as_str()) {
                    edges.insert((producer, index));
                }
            }
            if let Some(target) = node.render_target() {
                if let Some(&producer) = output_nodes.get(target) {
                    edges.insert((producer, index));
                }
            }
            for dependency in node.run_after() {
                if let Some(&producer) = name_map.get(dependency.as_str()) {
                    edges.insert((producer, index));
                }
            }
        }
        for (before, after) in &self.order_edges {
            if let (Some(&before), Some(&after)) =
                (name_map.get(before.as_str()), name_map.get(after.as_str()))
            {
                edges.insert((before, after));
            }
        }

        let mut in_degree = vec![0usize; node_count];
        let mut adjacency = vec![Vec::new(); node_count];
        for &(from, to) in &edges {
            adjacency[from].push(to);
            in_degree[to] += 1;
        }

        let mut queue: VecDeque<_> = (0..node_count)
            .filter(|&index| in_degree[index] == 0)
            .collect();
        let mut execution_order = Vec::with_capacity(node_count);
        while let Some(index) = queue.pop_front() {
            execution_order.push(index);
            for &dependent in &adjacency[index] {
                in_degree[dependent] -= 1;
                if in_degree[dependent] == 0 {
                    queue.push_back(dependent);
                }
            }
        }
        if execution_order.len() != node_count {
            comet_log::fatal!("Render graph contains a cycle");
        }

        let mut previous_textures = std::mem::take(&mut self.compiled_textures);
        let mut compiled_textures = Vec::new();
        let mut texture_slots = HashMap::new();
        for node in &self.nodes {
            let Some(output) = node.output() else {
                continue;
            };
            if texture_slots.contains_key(output) {
                continue;
            }
            let output_format = node.output_format().unwrap_or(format);
            let texture = previous_textures
                .iter()
                .position(|entry| {
                    entry.name == output
                        && entry.texture.size.width == width
                        && entry.texture.size.height == height
                        && entry.texture.texture.format() == output_format
                })
                .map(|position| previous_textures.swap_remove(position).texture)
                .unwrap_or_else(|| {
                    Arc::new(GpuTexture::create_2d_texture(
                        device,
                        width,
                        height,
                        output_format,
                        wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        wgpu::FilterMode::Linear,
                        Some(output),
                    ))
                });
            let slot = compiled_textures.len();
            texture_slots.insert(output.to_string(), slot);
            compiled_textures.push(CompiledTexture {
                name: output.to_string(),
                texture,
            });
        }

        let execution_plan = execution_order
            .into_iter()
            .map(|node_index| {
                let node = &self.nodes[node_index];
                let inputs = node
                    .inputs()
                    .iter()
                    .filter_map(|input| texture_slots.get(input).copied())
                    .map(|slot| compiled_textures[slot].texture.clone())
                    .collect();
                let target_name = node.render_target().or_else(|| node.output());
                let target = target_name
                    .and_then(|name| texture_slots.get(name).copied())
                    .map(|slot| compiled_textures[slot].texture.clone());
                CompiledNode {
                    node_index,
                    inputs,
                    target,
                    load: node.load_op(),
                    pass_label: format!("{} Pass", node.name()),
                }
            })
            .collect();

        self.compiled_textures = compiled_textures;
        self.execution_plan = execution_plan;
        self.dirty = false;
    }

    pub fn execute(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_view: &wgpu::TextureView,
        clear_color: wgpu::Color,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) {
        if self.dirty {
            self.compile(device, format, width, height);
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Graph Encoder"),
        });

        for compiled in &self.execution_plan {
            let load = match compiled.load {
                LoadOp::Background => wgpu::LoadOp::Clear(clear_color),
                LoadOp::Color(color) => wgpu::LoadOp::Clear(color),
                LoadOp::Load => wgpu::LoadOp::Load,
            };
            let view = compiled
                .target
                .as_ref()
                .map(|texture| &texture.view)
                .unwrap_or(surface_view);
            let color_attachment = Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load,
                    store: wgpu::StoreOp::Store,
                },
            });
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&compiled.pass_label),
                color_attachments: &[color_attachment],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            let state = NodeState {
                device,
                queue,
                inputs: &compiled.inputs,
                width,
                height,
            };
            self.nodes[compiled.node_index].run(&mut render_pass, &state);
        }

        queue.submit(std::iter::once(encoder.finish()));
        device.poll(wgpu::Maintain::Poll);
    }
}
