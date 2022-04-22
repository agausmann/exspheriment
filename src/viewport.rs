use std::f32::consts::TAU;

use bytemuck::{Pod, Zeroable};
use glam::{EulerRot, Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;

use crate::GraphicsContext;

const FOV: f32 = 75.0 / 360.0 * TAU;
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 1000.0;

#[derive(Debug, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    // mat4x4<f32>
    view_proj: [[f32; 4]; 4],

    // vec4<f32>
    camera: [f32; 3],
    z_near: f32,

    // vec4<f32>
    forward: [f32; 3],
    x_fov: f32,

    // vec4<f32>
    up: [f32; 3],
    y_fov: f32,
}

pub struct Viewport {
    gfx: GraphicsContext,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    pub camera_position: Vec3,
    pub up: Vec3,
    pub pitch: f32,
    pub yaw: f32,
}

impl Viewport {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let uniform_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Viewport::uniform_buffer"),
                contents: bytemuck::bytes_of(&Uniforms::zeroed()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Scene::bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT
                            | wgpu::ShaderStages::COMPUTE,
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
            camera_position: Vec3::new(0.0, -5.0, 3.0),
            up: Vec3::Z,
            pitch: 0.0,
            yaw: 0.0,
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn camera_pos(&self) -> Vec3 {
        self.camera_position
    }
    pub fn up(&self) -> Vec3 {
        self.camera_orientation() * Vec3::Z
    }

    pub fn forward(&self) -> Vec3 {
        self.camera_orientation() * Vec3::Y
    }

    pub fn camera_orientation(&self) -> Quat {
        Quat::from_euler(EulerRot::ZXY, self.yaw, self.pitch, 0.0)
    }

    pub fn aspect(&self) -> f32 {
        let size = self.gfx.window.inner_size();
        size.width as f32 / size.height as f32
    }

    pub fn view_proj(&self) -> Mat4 {
        let projection = Mat4::perspective_rh(FOV, self.aspect(), Z_NEAR, Z_FAR);
        let forward = self.camera_orientation() * Vec3::Y;
        let camera = Mat4::look_at_rh(
            self.camera_position,
            self.camera_position + forward,
            self.up(),
        );
        projection * camera
    }

    pub fn update(&mut self) {
        self.gfx.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&Uniforms {
                view_proj: self.view_proj().to_cols_array_2d(),
                camera: self.camera_position.into(),
                z_near: Z_NEAR,
                forward: self.forward().into(),
                up: self.up().into(),
                x_fov: ((FOV * 0.5).tan() * self.aspect()).atan() * 2.0,
                y_fov: FOV,
            }),
        );
    }
}
