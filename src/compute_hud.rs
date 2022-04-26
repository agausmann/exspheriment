use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::{viewport::Viewport, GraphicsContext};

const WORKGROUP_SIZE: u32 = 64;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Point {
    // vec4<f32>
    position: [f32; 3],
    size: f32,

    // vec4<f32>
    color: [f32; 4],
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Line {
    // vec4<f32>
    start: [f32; 3],
    size: f32,

    // vec3<f32>
    end: [f32; 3],
    _padding: [u8; 4],

    // vec4<f32>
    color: [f32; 4],
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Ellipse {
    // vec4<f32>
    center: [f32; 3],
    size: f32,

    // vec3<f32>
    axis_1: [f32; 3],
    _axis_1_padding: [u8; 4],

    // vec3<f32>
    axis_2: [f32; 3],
    _axis_2_padding: [u8; 4],

    // vec4<f32>
    color: [f32; 4],
}

pub struct Hud {
    gfx: GraphicsContext,
    points_buffer: wgpu::Buffer,
    lines_buffer: wgpu::Buffer,
    ellipses_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    point_pipeline: wgpu::ComputePipeline,
    line_pipeline: wgpu::ComputePipeline,
    ellipse_pipeline: wgpu::ComputePipeline,
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
                    color: [1.0, 1.0, 0.0, 1.0],
                }]),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let lines_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Hud::lines_buffer"),
                contents: bytemuck::cast_slice(&[Line {
                    start: [-1.0, 0.0, 3.0],
                    end: [1.0, 0.0, 3.0],
                    size: 5.0,
                    color: [0.5, 0.5, 0.5, 1.0],
                    _padding: Default::default(),
                }]),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let ellipses_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Hud::ellipses_buffer"),
                contents: bytemuck::cast_slice(&[Ellipse {
                    center: [0.0, 0.0, 3.0],
                    axis_1: [2.0, 0.0, 0.0],
                    axis_2: [0.0, -2.0, -1.0],
                    size: 3.0,
                    color: [0.0, 0.0, 1.0, 1.0],
                    _axis_1_padding: Default::default(),
                    _axis_2_padding: Default::default(),
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
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

        let point_pipeline = gfx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Hud::point_pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "point_main",
            });

        let line_pipeline = gfx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Hud::line_pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader_module,
                entry_point: "line_main",
            });

        let ellipse_pipeline =
            gfx.device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Hud::ellipse_pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &shader_module,
                    entry_point: "ellipse_main",
                });

        Self {
            gfx: gfx.clone(),
            points_buffer,
            lines_buffer,
            ellipses_buffer,
            bind_group_layout,
            point_pipeline,
            line_pipeline,
            ellipse_pipeline,
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
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.lines_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.ellipses_buffer.as_entire_binding(),
                    },
                ],
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Hud::compute_pass"),
            });
            let size = self.gfx.window.inner_size();
            pass.set_bind_group(0, viewport.bind_group(), &[]);
            pass.set_bind_group(1, &bind_group, &[]);

            pass.set_pipeline(&self.line_pipeline);
            pass.dispatch(div_ceil(size.width, WORKGROUP_SIZE), size.height, 1);

            pass.set_pipeline(&self.ellipse_pipeline);
            pass.dispatch(div_ceil(size.width, WORKGROUP_SIZE), size.height, 1);

            pass.set_pipeline(&self.point_pipeline);
            pass.dispatch(div_ceil(size.width, WORKGROUP_SIZE), size.height, 1);
        }
    }
}

fn div_ceil(a: u32, b: u32) -> u32 {
    (a - 1) / b + 1
}
