@group(0) @binding(0)
var t_screen: texture_2d<f32>;
@group(0) @binding(1)
var s_screen: sampler;

@group(1) @binding(0) var<uniform> time: f32;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let screen = textureSample(t_screen, s_screen, in.tex_coords);
    // return vec4f(abs(sin(time)-screen.x), abs(cos(time)-screen.y), screen.zw);
    return textureSample(t_screen, s_screen, in.tex_coords);
}