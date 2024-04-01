use bespoke_engine::model::{Model, Render};
use cgmath::{Quaternion, Vector3};
use wgpu::Device;

use crate::{game::Vertex, instance::Instance};

pub struct Billboard {
    model: Model,
    position: Vector3<f32>,
    rotation: Quaternion<f32>,
}

impl Billboard {
    pub fn new(width: f32, height: f32, size: f32, position: Vector3<f32>, rotation: Quaternion<f32>, device: &Device) -> Self {
        let vertices = vec![
            Vertex { position: [size*-width/2.0, size*-height/2.0, 0.0], tex_pos: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
            Vertex { position: [size*-width/2.0, size*height/2.0, 0.0], tex_pos: [0.0, 0.0], normal: [0.0, 0.0, 0.0] },
            Vertex { position: [size*width/2.0, size*-height/2.0, 0.0], tex_pos: [1.0, 1.0], normal: [0.0, 0.0, 0.0] },
            Vertex { position: [size*width/2.0, size*height/2.0, 0.0], tex_pos: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
        ];
        let model = Model::new_instances(vertices, &[0_u16, 1, 2, 2, 1, 3], vec![Instance {position, rotation}], device);
        Self {
            model,
            position,
            rotation,
        }
    }

    pub fn set_position(&mut self, position: Vector3<f32>, device: &Device) {
        self.position = position;
        self.create_instance(device);
    }

    pub fn set_rotation(&mut self, rotation: Quaternion<f32>, device: &Device) {
        self.rotation = rotation;
        self.create_instance(device);
    }

    pub fn set_both(&mut self, position: Vector3<f32>, rotation: Quaternion<f32>, device: &Device) {
        self.position = position;
        self.rotation = rotation;
        self.create_instance(device);
    }

    fn create_instance(&mut self, device: &Device) {
        self.model.update_instances(vec![Instance {position: self.position, rotation: self.rotation}], device);
    }
}

impl Render for Billboard {
    fn render<'a: 'b, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>) {
        self.model.render(render_pass);
    }
    fn render_instances<'a: 'b, 'c: 'b, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>, instances: &'c wgpu::Buffer, range: std::ops::Range<u32>) {
        self.model.render_instances(render_pass, instances, range);
    }
}