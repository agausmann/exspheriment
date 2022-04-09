use std::{f32::consts::TAU, ops::Range, time::Instant};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use once_cell::sync::Lazy;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{
    geometry::{Icosahedron, Square, Triangle},
    model::{self, Model},
    GraphicsContext,
};

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
    camera: [f32; 3],
    _padding: [u8; 4],
}

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Instance {
    model: [[f32; 4]; 4],
}

static INSTANCE_ATTRIBUTES: Lazy<[wgpu::VertexAttribute; 4]> = Lazy::new(|| {
    wgpu::vertex_attr_array![
        2 => Float32x4,
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
    ]
});

pub struct Scene {
    gfx: GraphicsContext,
    icos: Icosahedron,
    triangle: Triangle,
    square: Square,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    camera_position: Vec3,
    instances: Vec<Instance>,
    animation_start: Instant,
}

impl Scene {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let icos = Icosahedron::new(gfx);
        let triangle = Triangle::new(gfx);
        let square = Square::new(gfx);

        let instances = vec![Default::default(); 2];

        let uniform_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Scene::uniform_buffer"),
                contents: bytemuck::bytes_of(&Uniforms::default()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let instance_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Scene::instance_buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
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

        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Scene::pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader_module = gfx
            .device
            .create_shader_module(&include_wgsl!("scene.wgsl"));

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Scene::pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<model::Vertex>() as _,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &*model::VERTEX_ATTRIBUTES,
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: std::mem::size_of::<Instance>() as _,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &*INSTANCE_ATTRIBUTES,
                        },
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None, //Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gfx.depth_format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: gfx.render_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                multiview: None,
            });

        Self {
            gfx: gfx.clone(),
            icos,
            triangle,
            square,
            pipeline,
            bind_group,
            uniform_buffer,
            instance_buffer,
            instances,
            camera_position: Vec3::new(0.0, -3.0, 1.0),
            animation_start: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        // Square
        self.instances[0].model = Mat4::IDENTITY.to_cols_array_2d();

        //Triangle
        let t = self.animation_start.elapsed().as_secs_f32();
        self.instances[1].model = Mat4::from_scale_rotation_translation(
            Vec3::splat(0.5),
            Quat::from_rotation_z(TAU * t / 5.0),
            Vec3::new(0.0, 0.0, 1.5),
        )
        .to_cols_array_2d();
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        let size = self.gfx.window.inner_size();
        let projection = Mat4::perspective_infinite_rh(
            75.0_f32.to_radians(),
            size.width as f32 / size.height as f32,
            0.1,
        );
        let camera = Mat4::look_at_rh(self.camera_position, Vec3::new(0.0, 0.0, 1.0), Vec3::Z);
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
        self.gfx.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_model(&self.square.model, 0..1);
            // render_pass.draw_model(&self.triangle.model, 1..2);
            render_pass.draw_model(&self.icos.model, 1..2);
        }
    }
}

trait RenderPassExt<'a> {
    fn draw_model(&mut self, model: &'a Model, instances: Range<u32>);
}

impl<'a> RenderPassExt<'a> for wgpu::RenderPass<'a> {
    fn draw_model(&mut self, model: &'a Model, instances: Range<u32>) {
        self.set_vertex_buffer(0, model.vertex_buffer.slice(..));
        self.set_index_buffer(model.index_buffer.slice(..), model::INDEX_FORMAT);
        self.draw_indexed(model.index_range.clone(), 0, instances)
    }
}
