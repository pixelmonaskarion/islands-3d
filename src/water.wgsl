@group(0) @binding(0) var<uniform> camera: mat4x4<f32>;
@group(1) @binding(0) var<uniform> time: f32;


@group(2) @binding(0)
var t_water_normal: texture_2d<f32>;
@group(2) @binding(1)
var s_water_normal: sampler;

@group(3) @binding(0)
var t_water_normal2: texture_2d<f32>;
@group(3) @binding(1)
var s_water_normal2: sampler;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) tex_pos: vec2f,
    @location(2) normal: vec3f,
};
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_pos: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.clip_position = camera * model_matrix * vec4f(model.position, 1.0);
    out.tex_pos = model.tex_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4f(
        0.5 * dot(textureSample(t_water_normal, s_water_normal, in.tex_pos+normalize(vec2f(1.0, 1.0))*(time/10.0)).xyz, vec3(0.0, 1.0, 0.0))
        + dot(textureSample(t_water_normal2, s_water_normal2, in.tex_pos/5.0+vec2f(-1.0, 0.0)*(time/20.0)).xyz, vec3(0.0, 1.0, 0.0))
        + vec3f(0.0, 0.5, 1.0),
    0.5);
}