use std::{f32::consts::TAU, ops::Range, time::Instant};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use once_cell::sync::Lazy;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{
    geometry::{Geodesic, Square, Triangle},
    model::{self, Model},
    orbit::Orbit,
    viewport::Viewport,
    GraphicsContext,
};

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Instance {
    model: [[f32; 4]; 4],
    albedo: [f32; 3],
}

static INSTANCE_ATTRIBUTES: Lazy<[wgpu::VertexAttribute; 5]> = Lazy::new(|| {
    wgpu::vertex_attr_array![
        2 => Float32x4,
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x3,
    ]
});

pub struct Scene {
    gfx: GraphicsContext,
    triangle: Triangle,
    geodesic: Geodesic,
    square: Square,
    pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    animation_start: Instant,
    ellipse: Orbit,
    parabola: Orbit,
    hyperbola: Orbit,
}

impl Scene {
    pub fn new(gfx: &GraphicsContext, viewport: &Viewport) -> Self {
        let triangle = Triangle::new(gfx);
        let square = Square::new(gfx);
        let geodesic = Geodesic::with_slerp(gfx, 4);
        let parabola = Orbit::new(1.0, 2.0, 3.0);
        let ellipse = Orbit::new(0.5, 2.0, 3.0);
        let hyperbola = Orbit::new(1.5, 2.0, 3.0);

        let instances = vec![Default::default(); 5];

        let instance_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Scene::instance_buffer"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Scene::pipeline_layout"),
                bind_group_layouts: &[viewport.bind_group_layout()],
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
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: gfx.depth_format,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
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
            triangle,
            square,
            geodesic,
            pipeline,
            instance_buffer,
            instances,
            animation_start: Instant::now(),
            ellipse,
            parabola,
            hyperbola,
        }
    }

    pub fn update(&mut self) {
        let t = self.animation_start.elapsed().as_secs_f64();

        // Square
        self.instances[0].model = Mat4::IDENTITY.to_cols_array_2d();
        self.instances[0].albedo = Vec3::new(0.3, 0.6, 0.9).into();

        //Rotating icosahedron
        self.instances[1].model = Mat4::from_scale_rotation_translation(
            Vec3::splat(0.8),
            Quat::from_rotation_z(TAU * t as f32 / 20.0),
            Vec3::new(0.0, 0.0, 1.5),
        )
        .to_cols_array_2d();
        self.instances[1].albedo = Vec3::new(0.3, 0.6, 0.9).into();

        // Orbiting triangles
        let p = self.ellipse.current_position(t - 2.0);
        self.instances[2].model = Mat4::from_scale_rotation_translation(
            Vec3::splat(0.1),
            Quat::from_rotation_x(-TAU / 4.0),
            p.position.as_vec2().extend(1.5),
        )
        .to_cols_array_2d();
        self.instances[2].albedo = Vec3::new(0.9, 0.1, 0.2).into();

        let p = self.parabola.current_position(t - 2.0);
        self.instances[3].model = Mat4::from_scale_rotation_translation(
            Vec3::splat(0.1),
            Quat::from_rotation_x(-TAU / 4.0),
            p.position.as_vec2().extend(1.5),
        )
        .to_cols_array_2d();
        self.instances[3].albedo = Vec3::new(0.2, 0.9, 0.1).into();

        let p = self.hyperbola.current_position(t - 2.0);
        self.instances[4].model = Mat4::from_scale_rotation_translation(
            Vec3::splat(0.1),
            Quat::from_rotation_x(-TAU / 4.0),
            p.position.as_vec2().extend(1.5),
        )
        .to_cols_array_2d();
        self.instances[4].albedo = Vec3::new(0.1, 0.2, 0.9).into();
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        viewport: &Viewport,
    ) {
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
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, viewport.bind_group(), &[]);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_model(&self.square.model, 0..1);
            render_pass.draw_model(&self.geodesic.model, 1..2);
            render_pass.draw_model(&self.triangle.model, 2..5);
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
