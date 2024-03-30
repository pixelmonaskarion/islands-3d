use bespoke_engine::{binding::{Descriptor, UniformBinding}, model::{Model, Render, ToRaw}, texture::Texture};
use bespoke_engine::compute::ComputeShader;
use bytemuck::{bytes_of, NoUninit};
use cgmath::{Deg, InnerSpace, Quaternion, Rotation3, Vector3};
use image::{DynamicImage, GenericImageView, ImageError};
use wgpu::{Device, Queue};

use crate::instance::Instance;

#[repr(C)]
#[derive(NoUninit, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn pos(&self) -> Vector3<f32> {
        return Vector3::new(self.position[0], self.position[1], self.position[2]);
    }
}

impl Descriptor for Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

impl ToRaw for Vertex {
    fn to_raw(&self) -> Vec<u8> {
        bytes_of(self).to_vec()
    }
}

pub struct HeightMap {
    pub image: DynamicImage,
    pub models: Vec<((u32, u32), Model)>,
    pub width: u32,
    pub height: u32,
    pub size: f32,
    pub height_multiplier: f32,
}

impl HeightMap {
    pub fn from_bytes(device: &Device, image_bytes: &[u8], res: u32, size: f32, chunks: u32, height_multiplier: f32, gen_normals: bool) -> Result<Self, ImageError> {
        let image = image::load_from_memory(image_bytes)?.grayscale();
        let width = image.width()/res;
        let height = image.height()/res;
        let mut models = Vec::new();
        for cx in 0..chunks {
            for cy in 0..chunks {
                let mut vertices = vec![];
                let mut indices = vec![];
                let extra_x = if cx == chunks-1 {
                    0
                } else {
                    1
                };
                let extra_y = if cy == chunks-1 {
                    0
                } else {
                    1
                };
                for x in 0..width/chunks+extra_x {
                    for y in 0..height/chunks+extra_y {
                        let px = x + (width/chunks)*cx;
                        let py = y + (height/chunks)*cy;
                        let v_height = image.get_pixel(px*res, py*res).0[0] as f32 / 255.0 * height_multiplier;
                        let mut color = [17.0/255.0,124.0/255.0,19.0/255.0];
                        if v_height > height_multiplier*0.7 {
                            color = [0.9, 0.9, 0.9];
                        }
                        if v_height <= 0.1439215686*height_multiplier {
                            color = [0.3, 0.3, 0.3];
                        }
                        vertices.push(Vertex { position: [(px*res) as f32 * size, v_height, (py*res) as f32 * size], color, normal: [0.0, 1.0, 0.0] });
                        if x < (width/chunks+extra_x)-1 && y < (height/chunks+extra_y)-1 {
                            let i = x * (height/chunks+extra_y) + y;
                            indices.append(&mut [i, i+1, i+(height/chunks+extra_y)+1, i, i+(height/chunks+extra_y)+1, i+(height/chunks+extra_y)].to_vec());
                        }
                    }
                }
                if gen_normals {
                    for i in 0..indices.len()/3 {
                        let v1 = indices[i*3] as usize;
                        let v2 = indices[i*3+1] as usize;
                        let v3 = indices[i*3+2] as usize;
        
                        let u = vertices[v2].pos()-vertices[v1].pos();
                        let v = vertices[v3].pos()-vertices[v1].pos();
        
                        let mut normal = Vector3::new(0.0, 0.0, 0.0);
                        normal.x = u.y*v.z - u.z*v.y;
                        normal.y = u.z*v.x - u.x*v.z;
                        normal.z = u.x*v.y - u.y*v.x;
                        normal = normal.normalize();
                        vertices[v1].normal = normal.into();
                        vertices[v2].normal = normal.into();
                        vertices[v3].normal = normal.into();
                        if normal.y < 0.5 {
                            let dirt_color = [165.0/255.0,42.0/255.0,42.0/255.0];
                            if vertices[v1].color != [0.9, 0.9, 0.9] { vertices[v1].color = dirt_color; } 
                            if vertices[v2].color != [0.9, 0.9, 0.9] { vertices[v2].color = dirt_color; } 
                            if vertices[v3].color != [0.9, 0.9, 0.9] { vertices[v3].color = dirt_color; } 
                        }
                    }
                }
                let model = Model::new_instances(vertices, &indices, vec![
                    Instance {position: Vector3::new(0.0, 0.0, 0.0), rotation: Quaternion::from_axis_angle(Vector3::unit_z(), Deg(0.0))},
                ], device);
                models.push(((cx, cy), model));
            }
        }
        
        Ok(Self {
            models,
            width: image.width(),
            height: image.height(),
            size,
            image,
            height_multiplier,
        })
    }

    pub fn from_bytes_compute(device: &Device, queue: &Queue, image_bytes: &[u8], res: u32, size: f32, height_multiplier: f32, gen_normals: bool) -> Result<Self, ImageError> {
        let image = image::load_from_memory(image_bytes)?;
        let width = image.width()/res;
        let height = image.height()/res;
        let mut indices = vec![];
        for x in 0..width {
            for y in 0..height {
                if x < width-1 && y < height-1 {
                    let i = x * height + y;
                    indices.append(&mut [i, i+1, i+height+1, i, i+height+1, i+height].to_vec());
                }
            }
        }

        let image_texture = Texture::from_bytes(device, queue, image_bytes, "Height Map", None).unwrap();
        let compute_binding_type = Some(wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        });
        let image_height_binding = UniformBinding::new(device, "Image Height", height, None);
        let res_binding = UniformBinding::new(device, "Resolution", res, None);
        let size_binding = UniformBinding::new(device, "World Size", size, None);
        let height_multiplier_binding = UniformBinding::new(device, "Height Multiplier", height_multiplier, None);
        let blank_vertices: &[u8] = &(0..width*height*std::mem::size_of::<Vertex>() as u32).map(|_| 0_u8).collect::<Vec<_>>();
        let output_binding = UniformBinding::new_bytes(device, "Vertices Output", blank_vertices, compute_binding_type);
        let compute_shader = ComputeShader::new(include_str!("height_gen.wgsl"), &[/*&image_texture.layout,*/ &image_height_binding.layout, &res_binding.layout, &size_binding.layout, &height_multiplier_binding.layout, &output_binding.layout], device);
        compute_shader.run(&[/*image_texture.binding, */image_height_binding.binding, res_binding.binding, size_binding.binding, height_multiplier_binding.binding, output_binding.binding], [width, height, 1], device, queue);
        let model = Model::new_vertex_buffer(output_binding.buffer, width*height, &indices, device);
        Ok(Self {
            models: vec![((0, 0), model)],
            width: image.width(),
            height: image.height(),
            size,
            image,
            height_multiplier,
        })
    }

    pub fn get_height_at(&self, x: f32, y: f32) -> f32 {
        let x = (x/self.size).clamp(0.0, self.width as f32);
        let y = (y/self.size).clamp(0.0, self.height as f32);
        let x_fract = x.fract();
        let y_fract = y.fract();
        let x = x.floor() as u32;
        let y = y.floor() as u32;
        let height0 = self.image.get_pixel(x, y).0[0] as f32 / 255.0 * self.height_multiplier;
        let height1 = self.image.get_pixel(x, y+1).0[0] as f32 / 255.0 * self.height_multiplier;
        let height2 = self.image.get_pixel(x+1, y).0[0] as f32 / 255.0 * self.height_multiplier;
        let height3 = self.image.get_pixel(x+1, y+1).0[0] as f32 / 255.0 * self.height_multiplier;
        let heightx1 = height0+(height1-height0)*x_fract;
        let heightx2 = height2+(height3-height2)*x_fract;
        return heightx1 + (heightx2-heightx1)*y_fract;
        
    }
}

impl Render for HeightMap {
    fn render<'a: 'b, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>) {
        for (_, model) in &self.models {
            model.render(render_pass);
        }
    }
}