use bespoke_engine::{binding::Descriptor, model::ToRaw};
use bytemuck::bytes_of;
use cgmath::{Deg, Quaternion, Rotation3, Vector3};

#[derive(Clone)]
pub struct BananaInstance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub color: [f32; 4],
}

impl BananaInstance {
    pub fn raw(&self) -> BananaInstanceRaw {
        BananaInstanceRaw {model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(), color: self.color }
    }
}

impl Default for BananaInstance {
    fn default() -> Self {
        Self { position: Vector3::new(0.0, 0.0, 0.0), rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0)), color: [1.0, 1.0, 1.0, 0.0] }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct BananaInstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
}

impl ToRaw for BananaInstance {
    fn to_raw(&self) -> Vec<u8> {
        let raw = self.raw();
        bytes_of(&raw).to_vec()
    }
}

impl Descriptor for BananaInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<BananaInstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}