use crate::render_context::RenderContext;

pub struct RenderPass {
    pub label: String,
    pub execute: Box<
        dyn Fn(String, &mut RenderContext, &mut wgpu::CommandEncoder, &wgpu::TextureView)
            + Send
            + Sync,
    >,
}

impl RenderPass {
    pub fn new(
        label: String,
        execute: Box<
            dyn Fn(String, &mut RenderContext, &mut wgpu::CommandEncoder, &wgpu::TextureView)
                + Send
                + Sync,
        >,
    ) -> Self {
        Self { label, execute }
    }
}

pub fn universal_execute(
    label: String,
    ctx: &mut RenderContext,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(format!("{} Render Pass", label.clone()).as_str()),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(ctx.clear_color()),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
    });

    render_pass.set_pipeline(&ctx.get_pipeline(label.clone()).unwrap());

    let groups = ctx.resources().get_bind_groups(&label).unwrap();
    for i in 0..groups.len() {
        render_pass.set_bind_group(i as u32, groups.get(i).unwrap(), &[]);
    }

    render_pass.set_vertex_buffer(
        0,
        ctx.get_batch(label.clone())
            .unwrap()
            .vertex_buffer()
            .slice(..),
    );

    render_pass.set_index_buffer(
        ctx.get_batch(label.clone())
            .unwrap()
            .index_buffer()
            .slice(..),
        wgpu::IndexFormat::Uint16,
    );

    render_pass.draw_indexed(
        0..ctx.get_batch(label.clone()).unwrap().num_indices(),
        0,
        0..1,
    );
}
