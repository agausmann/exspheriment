use std::{f32::consts as f32, f64::consts as f64, ops::Range, time::Instant};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use once_cell::sync::Lazy;
use wgpu::{include_wgsl, util::DeviceExt};

use crate::{
    geometry::{Geodesic, Square, Triangle},
    model::{self, Model},
    orbit::{Orbit2D, Orbit3D, State3D},
    time::{SimDuration, SimInstant},
    viewport::Viewport,
    GraphicsContext,
};

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Instance {
    pub model: [[f32; 4]; 4],
    pub albedo: [f32; 3],
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
    pub instances: Vec<Instance>,
    animation_start: Instant,
    pub orbit: Orbit3D,
    pub state: Option<State3D>,
}

impl Scene {
    pub fn new(gfx: &GraphicsContext, viewport: &Viewport) -> Self {
        let triangle = Triangle::new(gfx);
        let square = Square::new(gfx);
        let geodesic = Geodesic::with_slerp(gfx, 4);
        let orbit = Orbit3D::new(
            Orbit2D::new(
                0.5,
                2.0,
                SimInstant::epoch() + SimDuration::from_secs_f64(30.0),
                1.0,
            ),
            f64::TAU / 4.0,
            f64::TAU / 8.0,
            0.0,
        );

        let instances = vec![Default::default(); 3];

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
            orbit,
            state: None,
        }
    }

    pub fn update(&mut self, viewport: &Viewport) {
        let t = self.animation_start.elapsed();

        // Square
        self.instances[0].model = Mat4::IDENTITY.to_cols_array_2d();
        self.instances[0].albedo = Vec3::new(0.3, 0.6, 0.9).into();

        //Rotating icosahedron
        self.instances[1].model = Mat4::from_scale_rotation_translation(
            Vec3::splat(0.8),
            Quat::from_rotation_z(f32::TAU * t.as_secs_f32() / 20.0),
            Vec3::new(0.0, 0.0, 1.5),
        )
        .to_cols_array_2d();
        self.instances[1].albedo = Vec3::new(0.3, 0.6, 0.9).into();

        // Orbiting triangles
        let mut state = self.orbit.current_state(SimInstant::epoch() + t.into());
        let orientation = state.position.cross(state.velocity).normalize();
        state.velocity += orientation * 0.001;
        self.state = Some(state);
        self.orbit = Orbit3D::from_current_state(&state, self.orbit.shape().grav());
        self.instances[2].model = Mat4::from_scale_rotation_translation(
            Vec3::splat(0.1),
            Quat::from_rotation_arc(
                -Vec3::Y,
                (viewport.camera_pos() - state.position.as_vec3()).normalize(),
            ),
            state.position.as_vec3() + Vec3::new(0.0, 0.0, 1.5),
        )
        .to_cols_array_2d();
        self.instances[2].albedo = Vec3::new(0.9, 0.1, 0.2).into();
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
            render_pass.draw_model(&self.triangle.model, 2..3);
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
