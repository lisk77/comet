pub mod node;
pub mod nodes;

pub use node::{BuildContext, NodeState, RenderNode};
pub use nodes::{PassNode, PostProcessNode};

use crate::gpu_texture::GpuTexture;
use crate::render_pass::LoadOp;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

pub struct RenderGraph {
    nodes: Vec<Box<dyn RenderNode>>,
    order_edges: Vec<(String, String)>,
    execution_order: Vec<usize>,
    dirty: bool,
    intermediate_textures: HashMap<String, Arc<GpuTexture>>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            order_edges: Vec::new(),
            execution_order: Vec::new(),
            dirty: true,
            intermediate_textures: HashMap::new(),
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
        node.build(BuildContext { device, queue, format, width, height });
        self.nodes.push(Box::new(node));
        self.dirty = true;
    }

    pub fn remove_node(&mut self, name: &str) {
        if let Some(pos) = self.nodes.iter().position(|n| n.name() == name) {
            let node = self.nodes.remove(pos);
            if let Some(out) = node.output() {
                self.intermediate_textures.remove(out);
            }
        }
        self.dirty = true;
    }

    pub fn add_order_edge(&mut self, before: &str, after: &str) {
        self.order_edges.push((before.to_string(), after.to_string()));
        self.dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn has_node(&self, name: &str) -> bool {
        self.nodes.iter().any(|n| n.name() == name)
    }

    pub fn get_node_mut<T: RenderNode>(&mut self, name: &str) -> Option<&mut T> {
        self.nodes
            .iter_mut()
            .find(|n| n.name() == name)?
            .as_any_mut()
            .downcast_mut::<T>()
    }

    pub fn on_resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        self.intermediate_textures.clear();
        for node in &mut self.nodes {
            node.on_resize(device, queue, width, height);
        }
    }

    fn compile(&mut self) {
        let n = self.nodes.len();

        let output_map: HashMap<String, usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| node.output().map(|out| (out.to_string(), i)))
            .collect();

        let name_map: HashMap<String, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.name().to_string(), i))
            .collect();

        let mut edges: HashSet<(usize, usize)> = HashSet::new();

        for (i, node) in self.nodes.iter().enumerate() {
            for input in node.inputs() {
                if let Some(&producer) = output_map.get(input) {
                    edges.insert((producer, i));
                }
            }
            if let Some(rt) = node.render_target() {
                if let Some(&producer) = output_map.get(rt) {
                    edges.insert((producer, i));
                }
            }
            for before in node.run_after() {
                if let Some(&b) = name_map.get(before) {
                    edges.insert((b, i));
                }
            }
        }

        for (before, after) in &self.order_edges {
            if let (Some(&b), Some(&a)) = (name_map.get(before), name_map.get(after)) {
                edges.insert((b, a));
            }
        }

        let mut in_degree = vec![0usize; n];
        let mut adj: Vec<Vec<usize>> = vec![vec![]; n];
        for (from, to) in &edges {
            adj[*from].push(*to);
            in_degree[*to] += 1;
        }

        let mut queue: VecDeque<usize> =
            (0..n).filter(|&i| in_degree[i] == 0).collect();
        let mut order = Vec::with_capacity(n);
        while let Some(i) = queue.pop_front() {
            order.push(i);
            for &dep in &adj[i] {
                in_degree[dep] -= 1;
                if in_degree[dep] == 0 {
                    queue.push_back(dep);
                }
            }
        }

        if order.len() != n {
            comet_log::fatal!("Render graph contains a cycle");
        }

        self.execution_order = order;
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
            self.compile();
        }

        for node in &self.nodes {
            if let Some(output_name) = node.output() {
                if !self.intermediate_textures.contains_key(output_name) {
                    let fmt = node.output_format().unwrap_or(format);
                    let tex = GpuTexture::create_2d_texture(
                        device,
                        width,
                        height,
                        fmt,
                        wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        wgpu::FilterMode::Linear,
                        Some(output_name),
                    );
                    self.intermediate_textures
                        .insert(output_name.to_string(), Arc::new(tex));
                }
            }
        }

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Graph Encoder"),
            });

        let order = self.execution_order.clone();
        for node_idx in order {
            let input_names: Vec<String> = self.nodes[node_idx]
                .inputs()
                .into_iter()
                .map(|s| s.to_string())
                .collect();
            let rt_name: Option<String> =
                self.nodes[node_idx].render_target().map(|s| s.to_string());
            let output_name: Option<String> =
                self.nodes[node_idx].output().map(|s| s.to_string());
            let load = self.nodes[node_idx].load_op();

            let input_textures: Vec<Arc<GpuTexture>> = input_names
                .iter()
                .filter_map(|name| self.intermediate_textures.get(name.as_str()).cloned())
                .collect();

            let target_tex: Option<Arc<GpuTexture>> = rt_name
                .as_deref()
                .or(output_name.as_deref())
                .and_then(|name| self.intermediate_textures.get(name).cloned());

            let load_op = match load {
                LoadOp::Background => wgpu::LoadOp::Clear(clear_color),
                LoadOp::Color(c) => wgpu::LoadOp::Clear(c),
                LoadOp::Load => wgpu::LoadOp::Load,
            };

            {
                let view = target_tex
                    .as_ref()
                    .map(|t| &t.view)
                    .unwrap_or(surface_view);

                let mut rpass =
                    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some(&format!(
                            "{} Pass",
                            self.nodes[node_idx].name()
                        )),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: load_op,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                let state = NodeState {
                    device,
                    queue,
                    inputs: &input_textures,
                    width,
                    height,
                };

                self.nodes[node_idx].run(&mut rpass, &state);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        device.poll(wgpu::Maintain::Poll);
    }
}
