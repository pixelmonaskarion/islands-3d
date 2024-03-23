use std::time::{SystemTime, UNIX_EPOCH};

use bespoke_engine::{binding::{Descriptor, UniformBinding}, camera::Camera, model::{Render, ToRaw}, shader::Shader, texture::Texture, window::{WindowConfig, WindowHandler}};
use bytemuck::{bytes_of, NoUninit};
use cgmath::{Point3, Vector2, Vector3};
use wgpu::{Device, Queue, RenderPass, TextureFormat};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::KeyEvent, keyboard::{KeyCode, PhysicalKey::Code}};

use crate::{height_map::HeightMap, instance::Instance, water::Water};

pub struct Game {
    camera_binding: UniformBinding,
    camera: Camera,
    time_binding: UniformBinding,
    start_time: u128,
    water_shader: Shader,
    keys_down: Vec<KeyCode>,
    water: Water,
    water_normal_image: Texture,
    water_normal2_image: Texture,
    height_map: HeightMap,
    ground_shader: Shader,
}

#[repr(C)]
#[derive(NoUninit, Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_pos: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    #[allow(dead_code)]
    pub fn pos(&self) -> Vector3<f32> {
        return Vector3::new(self.position[0], self.position[1], self.position[2]);
    }
}

impl Descriptor for Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
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


impl Game {
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat, size: PhysicalSize<u32>) -> Self {
        let screen_size = [size.width as f32, size.height as f32];
        let height_map = HeightMap::from_bytes(device, include_bytes!("height.png"), 2, 1.0, 250.0, true).unwrap();
        let camera = Camera {
            eye: Point3::new(height_map.width as f32/2.0, height_map.height_multiplier/2.0, height_map.height as f32/2.0),
            aspect: screen_size[0] / screen_size[1],
            fovy: 70.0,
            znear: 0.1,
            zfar: 10000.0,
            ground: 0.0,
            sky: 0.0,
        };
        let camera_binding = UniformBinding::new(device, "Camera", camera.build_view_projection_matrix());
        let time_binding = UniformBinding::new(device, "Time", 0.0_f32);
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let water_normal_image = Texture::from_bytes(device, queue, include_bytes!("water_normal.png"), "Water Normal Image", Some(wgpu::FilterMode::Linear)).unwrap();
        let water_normal2_image = Texture::from_bytes(device, queue, include_bytes!("water_normal2.png"), "Water Normal Image 2", Some(wgpu::FilterMode::Linear)).unwrap();
        let water_shader = Shader::new(include_str!("water.wgsl"), device, format, &[&camera_binding.layout, &time_binding.layout, &water_normal_image.layout, &water_normal2_image.layout], &[Vertex::desc(), Instance::desc()]);
        let water = Water::new(device, height_map.width.max(height_map.height) as f32, 0.1439215686*height_map.height_multiplier, 10.0);
        let ground_shader = Shader::new(include_str!("ground.wgsl"), device, format, &[&camera_binding.layout, &time_binding.layout], &[crate::height_map::Vertex::desc()]);
        println!("height map has {} triangles", height_map.model.num_indices/3);
        
        Self {
            camera_binding,
            camera,
            time_binding,
            start_time,
            water_shader,
            keys_down: vec![],
            water,
            water_normal_image,
            water_normal2_image,
            height_map,
            ground_shader,
        }
    }
}

impl WindowHandler for Game {
    fn resize(&mut self, _device: &Device, new_size: Vector2<u32>) {
        self.camera.aspect = new_size.x as f32 / new_size.y as f32;
    }

    fn render<'a: 'b, 'b>(&'a mut self, device: &Device, render_pass: & mut RenderPass<'b>, delta: f64) {
        let speed = 0.02 * delta as f32;
        if self.keys_down.contains(&KeyCode::KeyW) {
            self.camera.eye += self.camera.get_walking_vec() * speed;
        }
        if self.keys_down.contains(&KeyCode::KeyS) {
            self.camera.eye -= self.camera.get_walking_vec() * speed;
        }
        if self.keys_down.contains(&KeyCode::KeyA) {
            self.camera.eye -= self.camera.get_right_vec() * speed;
        }
        if self.keys_down.contains(&KeyCode::KeyD) {
            self.camera.eye += self.camera.get_right_vec() * speed;
        }
        if self.keys_down.contains(&KeyCode::Space) {
            self.camera.eye += Vector3::unit_y() * speed;
        }
        if self.keys_down.contains(&KeyCode::ShiftLeft) {
            self.camera.eye -= Vector3::unit_y() * speed;
        }
        // self.camera.eye.y = self.height_map.get_height_at(self.camera.eye.x, self.camera.eye.z)+2.0;
        self.camera_binding.set_data(device, self.camera.build_view_projection_matrix());
        self.time_binding.set_data(device, (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()-self.start_time) as f32 / 1000.0);

        render_pass.set_pipeline(&self.ground_shader.pipeline);
        
        render_pass.set_bind_group(0, &self.camera_binding.binding, &[]);
        render_pass.set_bind_group(1, &self.time_binding.binding, &[]);
        
        self.height_map.model.render(render_pass);
        
        render_pass.set_pipeline(&self.water_shader.pipeline);
        
        render_pass.set_bind_group(2, &self.water_normal_image.binding, &[]);
        render_pass.set_bind_group(3, &self.water_normal2_image.binding, &[]);
        
        self.water.model.render(render_pass);
    }

    fn config(&self) -> WindowConfig {
        WindowConfig {
            background_color: None,
        }
    }

    fn mouse_moved(&mut self, _device: &Device, _mouse_pos: PhysicalPosition<f64>) {

    }
    
    fn input_event(&mut self, _device: &Device, input_event: &KeyEvent) {
        if let Code(code) = input_event.physical_key {
            if input_event.state.is_pressed() {
                if !self.keys_down.contains(&code) {
                    self.keys_down.push(code);
                }
            } else {
                if let Some(i) = self.keys_down.iter().position(|x| x == &code) {
                    self.keys_down.remove(i);
                }
            }
        }
    }
    
    fn mouse_motion(&mut self, _device: &Device, delta: (f64, f64)) {
        self.camera.ground += (delta.0 / 500.0) as f32;
        self.camera.sky -= (delta.1 / 500.0) as f32;
        self.camera.sky = self.camera.sky.clamp(std::f32::consts::PI*-0.499, std::f32::consts::PI*0.499);
    }
}