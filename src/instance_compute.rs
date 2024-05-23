use bespoke_engine::{binding::UniformBinding, compute::ComputeShader};
use wgpu::{util::DeviceExt, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, Device, Queue};

use crate::banana_instance::{BananaInstance, BananaInstanceRaw};

pub struct BananaInstances {
    blank_instances: Vec<BananaInstanceRaw>,
    collected_buffer: Buffer,
    pub collected: Vec<(u32, u32)>,
    pub num_bananas: [usize; 2],
    dst_layout: BindGroupLayout,
    shader: ComputeShader,
    bananas_height_binding: UniformBinding<u32>,
}

impl BananaInstances {
    pub fn new(num_bananas: [usize; 2], shader_source: &str, time_layout: &BindGroupLayout, image_layout: &BindGroupLayout, device: &Device) -> Self {
        let blank_instances: Vec<_> = vec![BananaInstance::default().raw(); num_bananas[0] * num_bananas[1]];
        let dst_layout = 
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        // We will change the values in this buffer
                        read_only: false,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }, wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: true,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }]
        });
        let collected_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Collected Buffer")),
            contents: bytemuck::cast_slice::<u32, _>(&vec![0; num_bananas[0] * num_bananas[1]]),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let bananas_height_binding = UniformBinding::new(device, "Bananas Height", num_bananas[1] as u32, None);
        let compute_shader = ComputeShader::new(shader_source, &[&dst_layout, time_layout, image_layout, &bananas_height_binding.layout], device);
        Self {
            blank_instances,
            dst_layout,
            shader: compute_shader,
            collected_buffer,
            collected: Vec::new(),
            num_bananas,
            bananas_height_binding,
        }
    }

    pub fn collect(&mut self, pos: (u32, u32), device: &Device) {
        let i = pos.0 * 100 + pos.1;
        if i as usize >= self.num_bananas[0]*self.num_bananas[1] {
            return;
        }
        self.collected.push(pos);
        let mut collected_arr = vec![0_u32; self.num_bananas[0]*self.num_bananas[1]];
        for pos in &self.collected {
            let i = pos.0 * 100 + pos.1;
            collected_arr[i as usize] = 1;
        }
        self.collected_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Collected Buffer")),
            contents: bytemuck::cast_slice(&collected_arr),
            usage: wgpu::BufferUsages::STORAGE,
        });
    }
    
    pub fn create_bananas(&self, time_bind_group: &BindGroup, image_bind_group: &BindGroup, device: &Device, queue: &Queue) -> Buffer {
        let dst_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Output Vertex Buffer")),
            contents: bytemuck::cast_slice(&self.blank_instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
        });
        let dst_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.dst_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: dst_buffer.as_entire_binding(),
            }, BindGroupEntry {
                binding: 1,
                resource: self.collected_buffer.as_entire_binding(),
            }]
        });

        self.shader.run(&[&dst_bind_group, time_bind_group, image_bind_group, &self.bananas_height_binding.binding], [self.num_bananas[0] as u32, self.num_bananas[1] as u32, 1], device, queue);
        dst_buffer
    }
}