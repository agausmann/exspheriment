use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::GraphicsContext;

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    camera: [f32; 3],
    _padding: [u8; 4],
}

pub struct Viewport {
    gfx: GraphicsContext,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    camera_position: Vec3,
    look_at: Vec3,
}

impl Viewport {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let uniform_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Viewport::uniform_buffer"),
                contents: bytemuck::bytes_of(&Uniforms::default()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Scene::bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene::bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            gfx: gfx.clone(),
            bind_group_layout,
            bind_group,
            uniform_buffer,
            camera_position: Vec3::new(0.0, -0.5, 5.0),
            look_at: Vec3::new(0.0, 0.0, 1.5),
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn update(&mut self) {
        let size = self.gfx.window.inner_size();
        let projection = Mat4::perspective_rh(
            75.0_f32.to_radians(),
            size.width as f32 / size.height as f32,
            0.1,
            1000.0,
        );
        let camera = Mat4::look_at_rh(self.camera_position, self.look_at, Vec3::Z);
        let view_proj = projection * camera;

        self.gfx.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&Uniforms {
                view_proj: view_proj.to_cols_array_2d(),
                camera: self.camera_position.into(),
                ..Default::default()
            }),
        );
    }
}
