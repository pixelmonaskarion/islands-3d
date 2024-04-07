use std::{collections::HashMap, path::Path, time::{SystemTime, UNIX_EPOCH}};

use bespoke_engine::{binding::{Descriptor, UniformBinding}, camera::Camera, mesh::MeshModel, model::{Model, Render, ToRaw}, shader::{Shader, ShaderConfig}, texture::Texture, window::{WindowConfig, WindowHandler}};
use bytemuck::{bytes_of, NoUninit};
use cgmath::{MetricSpace, Quaternion, Rotation, Vector2, Vector3};
use wgpu::{Buffer, Device, Queue, RenderPass, TextureFormat};
use wgpu_text::{glyph_brush::{ab_glyph::FontRef, OwnedSection, OwnedText}, BrushBuilder, TextBrush};
use winit::{dpi::{PhysicalPosition, PhysicalSize}, event::{KeyEvent, TouchPhase}, keyboard::{KeyCode, PhysicalKey::Code}};

use crate::{banana_instance::BananaInstance, billboard::Billboard, height_map::HeightMap, instance::Instance, instance_compute::BananaInstances, load_resource, load_resource_string, water::Water};

pub struct Game {
    camera_binding: UniformBinding,
    camera: Camera,
    screen_size: [f32; 2],
    time_binding: UniformBinding,
    start_time: u128,
    water_shader: Shader,
    keys_down: Vec<KeyCode>,
    water: Water,
    water_normal_image: Texture,
    water_normal2_image: Texture,
    height_map: HeightMap,
    ground_shader: Shader,
    touch_positions: HashMap<u64, PhysicalPosition<f64>>,
    moving_bc_finger: Option<u64>,
    baby_billboard: Billboard,
    baby_image: Texture,
    sun_shader: Shader,
    post_processing_shader: Shader,
    model_shader: Shader,
    banana_model: MeshModel,
    banana_instances_gen: BananaInstances,
    banana_instances: Buffer,
    height_map_texture: Texture,
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
        let height_image_bytes = load_resource("res/height.png").unwrap();
        let height_map_texture = Texture::from_bytes(device, queue, &height_image_bytes, "Height Map Texture", None).unwrap();
        // let height_map = HeightMap::from_bytes_compute(device, queue, &load_resource("res/height.png").unwrap(), &height_map_texture, 2, 1.0, 250.0, true).unwrap();
        let height_map = HeightMap::make_data(&height_image_bytes, 2, 1.0, 10, 250.0, true).unwrap();
        let camera = Camera {
            eye: Vector3::new(height_map.width as f32/2.0, height_map.height_multiplier/2.0, height_map.height as f32/2.0),
            // eye: Vector3::new(0.0, 0.0, 0.0),
            aspect: screen_size[0] / screen_size[1],
            fovy: 70.0,
            znear: 0.1,
            zfar: 10000.0,
            ground: 0.0,
            sky: 0.0,
        };
        let camera_binding = UniformBinding::new(device, "Camera", camera.build_view_projection_matrix(), None);
        let time_binding = UniformBinding::new(device, "Time", 0.0_f32, None);
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let water_normal_image = Texture::from_bytes(device, queue, &load_resource("res/water_normal.png").unwrap(), "Water Normal Image", Some(wgpu::FilterMode::Linear)).unwrap();
        let water_normal2_image = Texture::from_bytes(device, queue, &load_resource("res/water_normal2.png").unwrap(), "Water Normal Image 2", Some(wgpu::FilterMode::Linear)).unwrap();
        let water_shader = Shader::new(include_str!("water.wgsl"), device, format, &[&camera_binding.layout, &time_binding.layout, &water_normal_image.layout, &water_normal2_image.layout], &[Vertex::desc(), Instance::desc()], None);
        let water = Water::new(device, height_map.width.max(height_map.height) as f32, 0.1439215686*height_map.height_multiplier, 10.0);
        let ground_shader = Shader::new(include_str!("ground.wgsl"), device, format, &[&camera_binding.layout, &time_binding.layout], &[crate::height_map::Vertex::desc(), Instance::desc()], None);
        let baby_image = Texture::from_bytes(device, queue, &load_resource("res/baby.png").unwrap(), "Baby Sun Image", Some(wgpu::FilterMode::Linear)).unwrap();
        let baby_dim = baby_image.normalized_dimensions();
        let position = camera.eye+Vector3::new(1.0_f32, 0.0, 0.0);
        let rotation = Quaternion::look_at(camera.eye-position, Vector3::new(0.0, 1.0, 0.0));
        let baby_billboard = Billboard::new(baby_dim.0, baby_dim.1, 1.0, position, rotation, device);
        let sun_shader = Shader::new(include_str!("billboard.wgsl"), device, format, &[&camera_binding.layout, &baby_image.layout], &[Vertex::desc(), Instance::desc()], Some(ShaderConfig {background: Some(false)}));
        let post_processing_shader = Shader::new_post_process(include_str!("post_process.wgsl"), device, format, &[&Texture::layout(device, None, None), &Texture::depth_layout(device, None, None), &time_binding.layout]);
        let model_texture_layout = Texture::layout(device, None, Some("Cube Material".into()));
        let model_shader = Shader::new(include_str!("model.wgsl"), device, format, &[&Texture::layout(device, None, None), &camera_binding.layout], &[Vertex::desc(), BananaInstance::desc()], None);
        let banana_model = MeshModel::load_model(Some("Cube".to_string()), Path::new("res/Banana_OBJ/Banana.obj"), load_resource_string, load_resource, device, queue, &model_texture_layout).unwrap();
        let banana_instances_gen = BananaInstances::new([100, 100], include_str!("banana_instances.wgsl"), &time_binding.layout, &height_map_texture.layout, device);
        let banana_instances = banana_instances_gen.create_bananas(&time_binding.binding, &height_map_texture.binding, device, queue);
        let text_brush = BrushBuilder::using_font_bytes(load_resource("res/ComicSansMS.ttf").unwrap()).unwrap()
            .build(&device, size.width, size.height, format);
        let text_section = OwnedSection::default().add_text(OwnedText::new(format!("0")).with_scale(200.0)
            .with_color([0.0, 0.7490196078, 1.0, 1.0]));
        Self {
            camera_binding,
            camera,
            screen_size,
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

    fn render<'a: 'b, 'b>(&'a mut self, device: &Device, queue: &Queue, render_pass: & mut RenderPass<'b>, delta: f64) {
        if self.height_map.models.is_some() {
            let speed = 1.0 * delta as f32;
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
                    self.banana_instances_gen.collect(banana_coords, device);
                    self.text_section.text = vec![OwnedText::new(self.banana_instances_gen.collected.len().to_string()).with_scale(200.0)
                    .with_color([0.0, 0.7490196078, 1.0, 1.0])];
                }
            }
            self.camera_binding.set_data(device, self.camera.build_view_projection_matrix());
            let time = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis()-self.start_time) as f32 / 1000.0;
            self.time_binding.set_data(device, time);
            let position = self.camera.eye+Vector3::new(time.cos(), time.sin(), 0.0);
            let rotation = Quaternion::look_at(self.camera.eye-position, Vector3::new(0.0, 1.0, 0.0));
            self.baby_billboard.set_both(position, rotation, device);

            render_pass.set_pipeline(&self.sun_shader.pipeline);
            
            render_pass.set_bind_group(0, &self.camera_binding.binding, &[]);
            render_pass.set_bind_group(1, &self.baby_image.binding, &[]);

            self.baby_billboard.render(render_pass);

            render_pass.set_pipeline(&self.ground_shader.pipeline);
            
            render_pass.set_bind_group(1, &self.time_binding.binding, &[]);
            
            self.height_map.render(render_pass);

            render_pass.set_pipeline(&self.model_shader.pipeline);
            render_pass.set_bind_group(1, &self.camera_binding.binding, &[]);
            self.banana_instances = self.banana_instances_gen.create_bananas(&self.time_binding.binding, &self.height_map_texture.binding, device, queue);
            self.banana_model.render_instances(render_pass, &self.banana_instances, 0..(self.banana_instances_gen.num_bananas[0]*self.banana_instances_gen.num_bananas[1]) as u32);

            render_pass.set_pipeline(&self.water_shader.pipeline);
            render_pass.set_bind_group(0, &self.camera_binding.binding, &[]);
            render_pass.set_bind_group(1, &self.time_binding.binding, &[]);
            render_pass.set_bind_group(2, &self.water_normal_image.binding, &[]);
            render_pass.set_bind_group(3, &self.water_normal2_image.binding, &[]);
            
            self.water.model.render(render_pass);
        } else {
            self.height_map.create_models(device);
        }
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
    
    fn post_process_render<'a: 'b, 'c: 'b, 'b>(&'a mut self, device: &Device, queue: &Queue, render_pass: & mut RenderPass<'b>, screen_model: &'c Model, surface_texture: &'c Texture, depth_texture: &'c Texture) {
        render_pass.set_pipeline(&self.post_processing_shader.pipeline);
        render_pass.set_bind_group(0, &surface_texture.binding, &[]);
        render_pass.set_bind_group(1, &depth_texture.binding, &[]);
        render_pass.set_bind_group(2, &self.time_binding.binding, &[]);

        screen_model.render(render_pass);
        self.text_brush.queue(device, queue, vec![&self.text_section]).unwrap();
        self.text_brush.draw(render_pass);
    }
}