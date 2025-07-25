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

// A simple pseudo-random number generator (using fragment coordinates)
fn rand2D(p: vec2<f32>) -> f32 {
    let s = sin(dot(p, vec2<f32>(12.9898, 78.233)));  // Pseudorandom calculation
    return fract(s * 43758.5453);  // Return value between 0 and 1
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture using the input texture coordinates
    var texColor: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Random horizontal displacement for glitching effect
    let glitchStrength = 0.03;  // How far the texture will "glitch"
    let glitchAmount = rand2D(in.tex_coords * 100.0);  // Use a different scale for more variation
    var displacedUV = in.tex_coords + vec2<f32>(glitchAmount * glitchStrength, 0.0);

    // Sample the texture at the displaced position
    texColor = textureSample(t_diffuse, s_diffuse, displacedUV);

    // Apply random color flickering
    let colorFlickerAmount = rand2D(in.tex_coords * 50.0);  // More frequency for faster flickering
    texColor.r *= mix(0.7, 1.3, colorFlickerAmount);  // Randomly adjust red channel brightness
    texColor.g *= mix(0.7, 1.3, colorFlickerAmount);  // Randomly adjust green channel brightness
    texColor.b *= mix(0.7, 1.3, colorFlickerAmount);  // Randomly adjust blue channel brightness

    // Occasionally "flicker" the texture to simulate complete signal loss
    let flickerChance = rand2D(in.tex_coords * 200.0);  // Different scale for randomness
    if (flickerChance < 0.05) {  // 5% chance to completely "flicker" out
        texColor.r = 0.0;  // Turn red channel to black
        texColor.g = 0.0;  // Turn green channel to black
        texColor.b = 0.0;  // Turn blue channel to black
    }

    // Apply random horizontal offset to simulate screen tearing
    let tearEffect = rand2D(in.tex_coords * 25.0);  // Vary this value for different effects
    if (tearEffect < 0.15) {
        texColor.r = 1.0;  // Simulate red "tear"
        texColor.g = 0.0;  // No green in the tear
        texColor.b = 0.0;  // No blue in the tear
    } else if (tearEffect < 0.3) {
        texColor.r = 0.0;  // No red in the tear
        texColor.g = 1.0;  // Simulate green "tear"
        texColor.b = 0.0;  // No blue in the tear
    }

    // Optionally, you can add a "flickering noise" layer for additional effect
    let noiseAmount = rand2D(in.tex_coords * 500.0);  // Highly random noise for flickering
    texColor.r += noiseAmount * 0.1;  // Add small amount of random noise to red channel
    texColor.g += noiseAmount * 0.1;  // Add small amount of random noise to green channel
    texColor.b += noiseAmount * 0.1;  // Add small amount of random noise to blue channel

    // Return the altered texture color
    return texColor;
}
