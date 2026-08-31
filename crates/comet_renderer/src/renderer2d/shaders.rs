pub(super) const SPRITE_SHADER: &str = r#"
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
"#;

pub(super) const GIZMO_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

pub(super) const FONT_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) field: f32,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    out.field = model.position.z;
    out.clip_position = camera.view_proj * vec4<f32>(model.position.xy, 0.0, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

fn median(a: f32, b: f32, c: f32) -> f32 {
    return max(min(a, b), min(max(a, b), c));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sample_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if in.field < -0.5 {
        let dimensions = textureDimensions(t_diffuse);
        let texel = vec2<i32>(in.tex_coords * vec2<f32>(dimensions));
        sample_color = textureLoad(t_diffuse, texel, 0);
    }
    var coverage = sample_color.a;
    if in.field > 0.0 {
        let distance = median(sample_color.r, sample_color.g, sample_color.b);
        let dimensions = vec2<f32>(textureDimensions(t_diffuse));
        let unit_range = vec2<f32>(in.field) / dimensions;
        let screen_texel_size = vec2<f32>(1.0) / max(fwidth(in.tex_coords), vec2<f32>(0.000001));
        let screen_range = max(0.5 * dot(unit_range, screen_texel_size), 1.0);
        coverage = clamp(screen_range * (distance - 0.5) + 0.5, 0.0, 1.0);
    }
    return vec4<f32>(in.color.rgb, coverage * in.color.a);
}
"#;
