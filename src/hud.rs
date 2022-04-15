use std::{f32::consts as f32, f64::consts as f64, num::NonZeroU32};

use glam::{Vec2, Vec3, Vec4Swizzles};
use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::{
    orbit::{Orbit2D, Orbit3D},
    viewport::Viewport,
    GraphicsContext,
};

pub struct Hud {
    gfx: GraphicsContext,
    pixmap: Pixmap,
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    orbit: Orbit3D,
}

impl Hud {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let orbit = Orbit3D::new(
            Orbit2D::new(0.5, 2.0, 3.0),
            f64::TAU / 4.0,
            f64::TAU / 8.0,
            0.0,
        );

        let size = gfx.window.inner_size();
        let pixmap = Pixmap::new(size.width, size.height).unwrap();
        let texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Hud::texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        let view = texture.create_view(&Default::default());
        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Hud::sampler"),
            ..Default::default()
        });

        let bind_group_layout =
            gfx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Hud::bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hud::bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader_module = gfx
            .device
            .create_shader_module(&wgpu::include_wgsl!("hud.wgsl"));

        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Hud::pipeline_layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Hud::pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[wgpu::ColorTargetState {
                        format: gfx.render_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::all(),
                    }],
                }),
                multiview: None,
            });

        Self {
            gfx: gfx.clone(),
            pixmap,
            texture,
            view,
            sampler,
            bind_group_layout,
            bind_group,
            pipeline,
            orbit,
        }
    }

    pub fn resized(&mut self) {
        let size = self.gfx.window.inner_size();
        self.pixmap = Pixmap::new(size.width, size.height).unwrap();
        self.texture = self.gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Hud::texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        self.view = self.texture.create_view(&Default::default());

        self.bind_group = self
            .gfx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Hud::bind_group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&self.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
    }

    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        viewport: &Viewport,
    ) {
        let view_proj = viewport.view_proj();

        let width = self.pixmap.width() as f32;
        let height = self.pixmap.height() as f32;
        let scale = 0.5 * Vec2::new(width, -height);
        let translate = 0.5 * Vec2::new(width, height);

        let origin = (self.orbit.a_vector()
            * (self.orbit.shape().rp() / self.orbit.shape().a() - 1.0))
            .as_vec3()
            + Vec3::new(0.0, 0.0, 1.5);
        let a = self.orbit.a_vector().as_vec3();
        let b = self.orbit.b_vector().as_vec3();

        let mut path_builder = PathBuilder::new();

        for i in 0..100 {
            let k = (i as f32) / 100.0 * f32::TAU;
            let point = origin + a * k.cos() + b * k.sin();
            let clip = view_proj * point.extend(1.0);
            let normalized = clip.xy() / clip.w;
            let screen = normalized * scale + translate;

            if i == 0 {
                path_builder.move_to(screen.x, screen.y);
            } else {
                path_builder.line_to(screen.x, screen.y);
            }
        }
        path_builder.close();
        let path = path_builder.finish().unwrap();

        let mut paint = Paint::default();
        paint.set_color_rgba8(127, 96, 64, 192);
        let mut stroke = Stroke::default();
        stroke.width = 2.0;

        self.pixmap
            .stroke_path(&path, &paint, &stroke, Transform::identity(), None);

        self.gfx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            self.pixmap.data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(self.pixmap.width() * 4),
                rows_per_image: NonZeroU32::new(self.pixmap.height()),
            },
            wgpu::Extent3d {
                width: self.pixmap.width(),
                height: self.pixmap.height(),
                ..Default::default()
            },
        );
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
    }
}
