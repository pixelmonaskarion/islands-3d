use std::{collections::HashMap, path::Path, time::{SystemTime, UNIX_EPOCH}};

use bespoke_engine::{billboard::Billboard, binding::{create_layout, Descriptor, UniformBinding}, camera::Camera, instance::Instance, mesh::MeshModel, model::{Model, Render, ToRaw}, shader::{Shader, ShaderConfig}, texture::{DepthTexture, Texture}, window::{SurfaceContext, WindowConfig, WindowHandler}};
use bytemuck::{bytes_of, NoUninit};
use cgmath::{MetricSpace, Quaternion, Rotation, Vector2, Vector3};
use wgpu::{Buffer, Device, Limits, Queue, RenderPass, TextureFormat};
use wgpu_text::{glyph_brush::{ab_glyph::FontRef, OwnedSection, OwnedText}, BrushBuilder, TextBrush};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::{KeyEvent, TouchPhase}, keyboard::{KeyCode, PhysicalKey::Code}};

use crate::{banana_instance::BananaInstance, height_map::HeightMap, instance_compute::BananaInstances, load_resource, load_resource_string, water::Water};

pub struct Game {
    camera_binding: UniformBinding<[[f32; 4]; 4]>,
    camera_inverse_binding: UniformBinding<[[f32; 4]; 4]>,
    camera_pos_binding: UniformBinding<[f32; 3]>,
    camera: Camera,
    screen_size: [f32; 2],
    screen_info_binding: UniformBinding<[f32; 4]>,
    time_binding: UniformBinding<f32>,
    start_time: u128,
    water_shader: Shader,
    keys_down: Vec<KeyCode>,
    water: Water,
    water_normal_image: UniformBinding<Texture>,
    water_normal2_image: UniformBinding<Texture>,
    height_map: HeightMap,
    ground_shader: Shader,
    touch_positions: HashMap<u64, PhysicalPosition<f64>>,
    moving_bc_finger: Option<u64>,
    baby_billboard: Billboard,
    baby_image: UniformBinding<Texture>,
    sun_shader: Shader,
    post_processing_shader: Shader,
    model_shader: Shader,
    banana_model: MeshModel,
    banana_instances_gen: BananaInstances,
    banana_instances: Buffer,
    height_map_texture: UniformBinding<Texture>,
    text_brush: TextBrush<FontRef<'static>>,
    text_section: OwnedSection,
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
        let screen_info_binding = UniformBinding::new(device, "Screen Size", [screen_size[0], screen_size[1], 0.0, 0.0], None);
        let height_image_bytes = &load_resource("res/height.png").unwrap();
        let height_map_texture = UniformBinding::new(device, "Height Map Texture", Texture::from_bytes(device, queue, &height_image_bytes, "Height Map Texture", None).unwrap(), None);
        // let height_map = HeightMap::from_bytes_compute(device, queue, &load_resource("res/height.png").unwrap(), &height_map_texture.value, 2, 1.0, 250.0, true).unwrap();
        let height_map = HeightMap::from_bytes(device, height_image_bytes, 2, 1.0, 5, 250.0, true).unwrap();
        // let height_map = HeightMap::make_data(&height_image_bytes, 2, 1.0, 10, 250.0, true).unwrap();
        let camera = Camera {
            eye: Vector3::new(height_map.width as f32/2.0, height_map.height_multiplier/5.0, height_map.height as f32/2.0),
            // eye: Vector3::new(0.0, 0.0, 0.0),
            aspect: screen_size[0] / screen_size[1],
            fovy: 70.0,
            znear: 0.1,
            zfar: 100.0,
            ground: 0.0,
            sky: 0.0,
        };
        let camera_binding = UniformBinding::new(device, "Camera", camera.build_view_projection_matrix_raw(), None);
        let camera_inverse_binding = UniformBinding::new(device, "Camera Inverse", camera.build_inverse_matrix_raw(), None);
        let camera_pos_binding = UniformBinding::new(device, "Camera Position", Into::<[f32; 3]>::into(camera.eye), None);
        let time_binding = UniformBinding::new(device, "Time", 0.0_f32, None);
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let water_normal_image = UniformBinding::new(device, "Water Normal Texture", Texture::from_bytes(device, queue, &load_resource("res/water_normal.png").unwrap(), "Water Normal Image", Some(wgpu::FilterMode::Linear)).unwrap(), None);
        let water_normal2_image = UniformBinding::new(device, "Water Normal Texture 2", Texture::from_bytes(device, queue, &load_resource("res/water_normal2.png").unwrap(), "Water Normal Image 2", Some(wgpu::FilterMode::Linear)).unwrap(), None);
        let water_shader = Shader::new(include_str!("water.wgsl"), device, format, vec![&camera_binding.layout, &time_binding.layout, &water_normal_image.layout, &water_normal2_image.layout], &[Vertex::desc(), Instance::desc()], None);
        let water = Water::new(device, height_map.width.max(height_map.height) as f32, 0.1439215686*height_map.height_multiplier, 10.0);
        let ground_shader = Shader::new(include_str!("ground.wgsl"), device, format, vec![&camera_binding.layout, &time_binding.layout], &[crate::height_map::Vertex::desc(), Instance::desc()], Some(ShaderConfig {line_mode: Some(wgpu::PolygonMode::Fill), ..Default::default()}));
        let baby_image = UniformBinding::new(device, "Baby Texture", Texture::from_bytes(device, queue, &load_resource("res/baby.png").unwrap(), "Baby Sun Image", Some(wgpu::FilterMode::Linear)).unwrap(), None);
        let baby_dim = baby_image.value.normalized_dimensions();
        let position = camera.eye+Vector3::new(1.0_f32, 0.0, 0.0);
        let rotation = Quaternion::look_at(camera.eye-position, Vector3::new(0.0, 1.0, 0.0));
        let baby_billboard = Billboard::new(baby_dim.0, baby_dim.1, 1.0, position, rotation, device);
        let sun_shader = Shader::new(include_str!("billboard.wgsl"), device, format, vec![&camera_binding.layout, &baby_image.layout], &[Vertex::desc(), Instance::desc()], Some(ShaderConfig {background: Some(false), ..Default::default()}));
        let post_processing_shader = Shader::new_post_process(include_str!("post_process.wgsl"), device, format, &[&create_layout::<Texture>(device), &create_layout::<DepthTexture>(device), &screen_info_binding.layout, &camera_binding.layout, &camera_inverse_binding.layout, &camera_pos_binding.layout]);
        let model_texture = UniformBinding::new(device, "Model Texture", Texture::blank_texture(device, 1, 1, format), None);
        let model_shader = Shader::new(include_str!("model.wgsl"), device, format, vec![&model_texture.layout, &camera_binding.layout, &time_binding.layout], &[Vertex::desc(), BananaInstance::desc()], None);
        let banana_model = MeshModel::load_model(Some("Cube".to_string()), Path::new("res/Banana_OBJ/Banana.obj"), load_resource_string, load_resource, device, queue, &create_layout::<Texture>(device)).unwrap();
        let banana_instances_gen = BananaInstances::new([100, 100], include_str!("banana_instances.wgsl"), &time_binding.layout, &height_map_texture.layout, device);
        let banana_instances = banana_instances_gen.create_bananas(&time_binding.binding, &height_map_texture.binding, device, queue);
        let text_brush = BrushBuilder::using_font_bytes(load_resource("res/ComicSansMS.ttf").unwrap()).unwrap()
            .build(&device, size.width, size.height, format);
        let text_section = OwnedSection::default().add_text(OwnedText::new(format!("0")).with_scale(200.0)
            .with_color([0.0, 0.7490196078, 1.0, 1.0]));
        Self {
            camera_binding,
            camera_inverse_binding,
            camera_pos_binding,
            camera,
            screen_size,
            screen_info_binding,
            time_binding,
            start_time,
            water_shader,
            keys_down: vec![],
            water,
            water_normal_image,
            water_normal2_image,
            height_map,
            ground_shader,
            touch_positions: HashMap::new(),
            moving_bc_finger: None,
            baby_billboard,
            baby_image,
            sun_shader,
            post_processing_shader,
            model_shader,
            banana_model,
            banana_instances_gen,
            banana_instances,
            height_map_texture,
            text_brush,
            text_section,
        }
    }
}

impl WindowHandler for Game {
    fn resize(&mut self, _device: &Device, queue: &Queue, new_size: Vector2<u32>) {
        self.camera.aspect = new_size.x as f32 / new_size.y as f32;
        self.screen_size = [new_size.x as f32, new_size.y as f32];

        self.text_brush.resize_view(new_size.x as f32, new_size.y as f32, queue);
    }

    fn render<'s: 'b, 'b>(&'s mut self, surface_ctx: &SurfaceContext, render_pass: & mut RenderPass<'b>, delta: f64) {
        if self.height_map.models.is_some() {
            let speed = 0.02 * delta as f32;
            if self.keys_down.contains(&KeyCode::KeyW) || self.moving_bc_finger.is_some() {
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
            self.camera.eye.y = self.height_map.get_height_at(self.camera.eye.x, self.camera.eye.z)+2.0;
            let banana_coords = ((self.camera.eye.x/(30.96)).round() as u32, (self.camera.eye.z/(30.96)).round() as u32);
            if !self.banana_instances_gen.collected.contains(&banana_coords) {
                let dist = self.camera.eye.distance(Vector3::new(banana_coords.0 as f32 * 30.96, self.camera.eye.y, banana_coords.1 as f32 *30.96));
                if dist < 5.0 {
                    self.banana_instances_gen.collect(banana_coords, &surface_ctx.device);
                    self.text_section.text = vec![OwnedText::new(self.banana_instances_gen.collected.len().to_string()).with_scale(200.0)
                    .with_color([0.0, 0.7490196078, 1.0, 1.0])];
                }
            }
            self.camera_binding.set_data(&surface_ctx.device, self.camera.build_view_projection_matrix_raw());
            self.camera_inverse_binding.set_data(&surface_ctx.device, self.camera.build_inverse_matrix_raw());
            self.camera_pos_binding.set_data(&surface_ctx.device, Into::<[f32; 3]>::into(self.camera.eye));
            let time = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()-self.start_time) as f32 / 1000.0;
            self.time_binding.set_data(&surface_ctx.device, time);
            self.screen_info_binding.set_data(&surface_ctx.device, [self.screen_size[0], self.screen_size[1], time, 0.0]);
            let position = self.camera.eye+Vector3::new((time/10.0).cos(), (time/10.0).sin(), 0.0);
            let rotation = Quaternion::look_at(self.camera.eye-position, Vector3::new(0.0, 1.0, 0.0));
            self.baby_billboard.set_both(position, rotation, &surface_ctx.device);

            self.sun_shader.bind(render_pass);
            
            render_pass.set_bind_group(0, &self.camera_binding.binding, &[]);
            render_pass.set_bind_group(1, &self.baby_image.binding, &[]);

            self.baby_billboard.render(render_pass);

            self.ground_shader.bind(render_pass);
            
            render_pass.set_bind_group(1, &self.time_binding.binding, &[]);
            
            self.height_map.render(render_pass);

            self.model_shader.bind(render_pass);
            render_pass.set_bind_group(1, &self.camera_binding.binding, &[]);
            render_pass.set_bind_group(2, &self.time_binding.binding, &[]);
            self.banana_instances = self.banana_instances_gen.create_bananas(&self.time_binding.binding, &self.height_map_texture.binding, &surface_ctx.device, &surface_ctx.queue);
            self.banana_model.render_instances(render_pass, &self.banana_instances, 0..(self.banana_instances_gen.num_bananas[0]*self.banana_instances_gen.num_bananas[1]) as u32);

            self.water_shader.bind(render_pass);
            render_pass.set_bind_group(0, &self.camera_binding.binding, &[]);
            render_pass.set_bind_group(1, &self.time_binding.binding, &[]);
            render_pass.set_bind_group(2, &self.water_normal_image.binding, &[]);
            render_pass.set_bind_group(3, &self.water_normal2_image.binding, &[]);
            
            self.water.model.render(render_pass);
        } else {
            self.height_map.create_models(&surface_ctx.device);
        }
    }

    fn config(&self) -> Option<WindowConfig> {
        Some(WindowConfig { background_color: None, enable_post_processing: Some(true) })
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
    
    fn touch(&mut self, device: &Device, touch: &winit::event::Touch) {
        match touch.phase {
            TouchPhase::Moved => {
                if let Some(last_position) = self.touch_positions.get(&touch.id) {
                    let delta = (touch.location.x-last_position.x, touch.location.y-last_position.y);
                    self.mouse_motion(device, delta);
                    self.touch_positions.insert(touch.id, touch.location);
                }
            }
            TouchPhase::Started => {
                if touch.location.x <= self.screen_size[0] as f64 / 2.0 {
                    self.touch_positions.insert(touch.id, touch.location);
                } else {
                    self.moving_bc_finger = Some(touch.id);
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.touch_positions.remove(&touch.id);
                if self.moving_bc_finger == Some(touch.id) {
                    self.moving_bc_finger = None;
                }
            }
        }
    }
    
    fn post_process_render<'s: 'b, 'c: 'b, 'b>(&'s mut self, device: &Device, queue: &Queue, render_pass: & mut RenderPass<'b>, screen_model: &'c Model, surface_texture: &'c UniformBinding<Texture>, depth_texture: &'c UniformBinding<DepthTexture>) {
        self.post_processing_shader.bind(render_pass);
        render_pass.set_bind_group(0, &surface_texture.binding, &[]);
        render_pass.set_bind_group(1, &depth_texture.binding, &[]);
        render_pass.set_bind_group(2, &self.screen_info_binding.binding, &[]);
        render_pass.set_bind_group(3, &self.camera_binding.binding, &[]);
        render_pass.set_bind_group(4, &self.camera_inverse_binding.binding, &[]);
        render_pass.set_bind_group(5, &self.camera_pos_binding.binding, &[]);

        screen_model.render(render_pass);
        self.text_brush.queue(device, queue, vec![&self.text_section]).unwrap();
        self.text_brush.draw(render_pass);
    }
    
    fn limits() -> wgpu::Limits {
        Limits {
            max_bind_groups: 6,
            ..Default::default()
        }
    }
    
    fn other_window_event(&mut self, _device: &Device, _queue: &Queue, _event: &winit::event::WindowEvent) {
        
    }
}