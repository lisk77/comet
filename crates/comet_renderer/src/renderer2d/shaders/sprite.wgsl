struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) position_size: vec4<f32>,
    @location(3) rotation: vec2<f32>,
    @location(4) uv_min: vec2<f32>,
    @location(5) uv_max: vec2<f32>,
    @location(6) instance_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let local = model.position.xy * model.position_size.zw;
    let rotated = vec2<f32>(
        local.x * model.rotation.x - local.y * model.rotation.y,
        local.x * model.rotation.y + local.y * model.rotation.x,
    );
    let world_position = model.position_size.xy + rotated;
    out.clip_position = camera.view_proj * vec4<f32>(world_position, model.position.z, 1.0);
    out.tex_coords = mix(model.uv_min, model.uv_max, model.tex_coords);
    out.color = model.instance_color;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords) * in.color;
}
