// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
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
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;  // Diffuse texture
@group(0) @binding(1)
var s_diffuse: sampler;  // Sampler for the texture

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture using the input texture coordinates
    var texColor: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Apply CRT curvature effect (distortion)
    let center = vec2<f32>(0.5, 0.5);  // center of the screen
    let dist = distance(in.tex_coords, center);  // Distance from the center
    let curvature = 0.1;  // Adjust for more or less curvature
    var distorted_uv = in.tex_coords + (in.tex_coords - center) * curvature * dist;

    // Apply chromatic aberration by slightly offsetting the UV coordinates
    let redOffset = 0.005;
    let greenOffset = 0.0;
    let blueOffset = -0.005;

    let redColor = textureSample(t_diffuse, s_diffuse, distorted_uv + vec2<f32>(redOffset, 0.0));
    let greenColor = textureSample(t_diffuse, s_diffuse, distorted_uv + vec2<f32>(greenOffset, 0.0));
    let blueColor = textureSample(t_diffuse, s_diffuse, distorted_uv + vec2<f32>(blueOffset, 0.0));

    // Combine chromatic aberration with the original color
    texColor.r = redColor.r;
    texColor.g = greenColor.g;
    texColor.b = blueColor.b;

    // Apply scanline effect (darken even rows for scanlines)
    let scanlineEffect = 0.1 * (sin(in.clip_position.y * 0.3) + 1.0);  // Horizontal scanlines
    texColor.r *= scanlineEffect;  // Apply scanline effect to red channel
    texColor.g *= scanlineEffect;  // Apply scanline effect to green channel
    texColor.b *= scanlineEffect;  // Apply scanline effect to blue channel

    return texColor;
}
