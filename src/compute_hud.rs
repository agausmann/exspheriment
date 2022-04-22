use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::{viewport::Viewport, GraphicsContext};

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Point {
    // vec4<f32>
    position: [f32; 3],
    size: f32,

    // vec4<f32>
    color: [f32; 4],
}

pub struct Hud {
    gfx: GraphicsContext,
    points_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl Hud {
    pub fn new(gfx: &GraphicsContext, viewport: &Viewport) -> Self {
        let points_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Hud::points_buffer"),
                contents: bytemuck::cast_slice(&[Point {
                    position: [0.0, 0.0, 3.0],
                    size: 10.0,
                    color: [1.0, 1.0, 0.0, 0.0],
                }]),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Hud::bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::WriteOnly,
                                format: gfx.render_format,
                                view_dimension: wgpu::TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let shader_module = gfx
            .device
            .create_shader_module(&wgpu::include_wgsl!("compute_hud.wgsl"));

        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Hud::pipeline_layout"),
                bind_group_layouts: &[viewport.bind_group_layout(), &bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gfx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Hud::compute_pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "point_main",
            });

        Self {
            gfx: gfx.clone(),
            points_buffer,
            bind_group_layout,
            pipeline,
        }
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        viewport: &Viewport,
    ) {
        let bind_group = self
            .gfx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Hud::bind_group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(frame_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.points_buffer.as_entire_binding(),
                    },
                ],
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Hud::compute_pass"),
            });
            let size = self.gfx.window.inner_size();
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, viewport.bind_group(), &[]);
            pass.set_bind_group(1, &bind_group, &[]);
            pass.dispatch(size.width, size.height, 1);
        }
    }
}
