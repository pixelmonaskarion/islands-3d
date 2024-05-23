@group(0) @binding(0)
var t_screen: texture_2d<f32>;
@group(0) @binding(1)
var s_screen: sampler;
@group(1) @binding(0)
var t_depth: texture_depth_2d;

struct ScreenInfo {
    screen_size: vec2f,
    time: f32,
}

@group(2) @binding(0) var<uniform> screen_info: ScreenInfo;
@group(3) @binding(0) var<uniform> camera: mat4x4<f32>;
@group(4) @binding(0) var<uniform> camera_inverse: mat4x4<f32>;
@group(5) @binding(0) var<uniform> camera_pos: vec3f;

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
    let screen = textureSample(t_screen, s_screen, in.tex_coords.xy);
    let depth_value = textureLoad(t_depth, vec2<u32>(u32(in.tex_coords.x*screen_info.screen_size.x), u32(in.tex_coords.y*screen_info.screen_size.y)), 0);
    if depth_value == 1.0 {
        var z = 0.1;
        let clipPos = vec4(in.tex_coords.x * 2.0 - 1.0, in.tex_coords.y * -2.0 + 1.0, z, 1.0);
        let viewPos = camera_inverse * clipPos;
        let worldPos = viewPos.xyz / viewPos.w;
        let diff = ((worldPos-camera_pos).y+0.3)*1.5;
        return vec4f(diff*0.1098039216, diff*0.4941176471, diff*0.9294117647, 1.0);
    } else {
        return screen;
    }
}