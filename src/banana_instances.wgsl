struct Instance {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) color: vec4<f32>,
};

@group(0)
@binding(0)
var<storage, read_write> dst_instances: array<Instance>;
@group(0)
@binding(1)
var<storage, read> collected: array<vec4<u32>>;

@group(1) @binding(0)
var<uniform> time: f32;

@group(2) @binding(0)
var t_height: texture_2d<f32>;
@group(2) @binding(1)
var s_height: sampler;

@group(3) @binding(0)
var<uniform> height: u32;
const HEIGHT_MAP_SIZE: f32 = 2.0;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let texture = textureLoad(t_height, vec2<u32>(u32(f32(global_id.x)*30.96), u32(f32(global_id.y)*30.96)), 0);
    let v_height = pow(texture.x, 0.4) * 250.0;
    let i = global_id.x * height + global_id.y;
    let matrix = mat4x4f(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )*quaternion_to_matrix(quaternion(vec3f(0.0, 1.0, 0.0), time*2.0));
    var instance: Instance;
    instance.model_matrix_0 = matrix[0];
    instance.model_matrix_1 = matrix[1];
    instance.model_matrix_2 = matrix[2];
    instance.model_matrix_3 = vec4f(f32(global_id.x)*30.96, v_height-10.0, f32(global_id.y)*30.96, 1.0);
    if collected[i/4][i % 4] != 0 {
        instance.color = vec4f(0.0, 0.7490196078, 1.0, 1.0);
    }
    dst_instances[i] = instance;
}

fn quaternion_to_matrix(quat: vec4f) -> mat4x4f {
    let x2 = quat.x + quat.x;
    let y2 = quat.y + quat.y;
    let z2 = quat.z + quat.z;

    let xx2 = x2 * quat.x;
    let xy2 = x2 * quat.y;
    let xz2 = x2 * quat.z;

    let yy2 = y2 * quat.y;
    let yz2 = y2 * quat.z;
    let zz2 = z2 * quat.z;

    let sy2 = y2 * quat.w;
    let sz2 = z2 * quat.w;
    let sx2 = x2 * quat.w;

    return mat4x4f(
        1.0 - yy2 - zz2, xy2 + sz2, xz2 - sy2, 0.0,
        xy2 - sz2, 1.0 - xx2 - zz2, yz2 + sx2, 0.0,
        xz2 + sy2, yz2 - sx2, 1.0 - xx2 - yy2, 0.0,
        0.0, 0.0, 0.0, 1.0,
    );
}

fn quaternion(axis: vec3f, angle: f32) -> vec4f {
    return vec4f(axis*sin(angle*0.5), cos(angle*0.5));
}