// @group(0) @binding(0)
// var t_height: texture_2d<f32>;
// @group(0) @binding(1)
// var s_height: sampler;

@group(0) @binding(0)
var<uniform> image_height: u32;
@group(1) @binding(0)
var<uniform> res: u32;
@group(2) @binding(0)
var<uniform> size: f32;
@group(3) @binding(0)
var<uniform> height_multiplier: f32;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

@group(4)
@binding(0)
var<storage, read_write> vertices: array<Vertex>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    var out: Vertex;
    out.position = vec3f(f32(global_id.x * res) * size, f32(global_id.x+global_id.y), f32(global_id.y * res) * size);
    // out.position = vec3f(1.0, 1.0, 1.0);
    out.color = vec3f(f32(global_id.x)/f32(image_height), f32(global_id.y)/f32(image_height), 0.0);
    out.normal = vec3f(1.0, 1.0, 1.0);
    vertices[global_id.x * image_height + global_id.y] = out;
}