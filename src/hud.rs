use std::{
    f32::consts as f32,
    f64::consts::{self as f64, TAU},
    num::NonZeroU32,
};

use glam::{DVec3, Vec2, Vec4Swizzles};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::{
    orbit::{Orbit2D, Orbit3D, State3D},
    time::SimInstant,
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
    pub orbit: Orbit3D,
    pub state: Option<State3D>,
}

impl Hud {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let orbit = Orbit3D::new(
            Orbit2D::new(0.5, 2.0, SimInstant::epoch(), 3.0),
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
            state: None,
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

        let map_3d = |point: DVec3| {
            let clip = view_proj * point.as_vec3().extend(1.0);
            let normalized = clip.xy() / clip.w;
            let screen = normalized * scale + translate;
            screen
        };

        self.pixmap.fill(Color::TRANSPARENT);

        let mut stroke = Stroke::default();
        let mut paint = Paint::default();
        stroke.width = 2.0;
        paint.anti_alias = true;

        // let focus = DVec3::new(0.0, 0.0, 1.5);
        // let center = (self.orbit.a_vector()
        //     * (self.orbit.shape().rp() / self.orbit.shape().a() - 1.0))
        //     + focus;
        // let a = self.orbit.a_vector();
        // let b = self.orbit.b_vector();

        // dbg!(&self.orbit);

        // let mut orbit_path = PathBuilder::new();
        // for i in 0..100 {
        //     let k = (i as f64) / 100.0 * TAU;
        //     let point = map_3d(center + a * k.cos() + b * k.sin());

        //     if i == 0 {
        //         orbit_path.move_to(point.x, point.y);
        //     } else {
        //         orbit_path.line_to(point.x, point.y);
        //     }
        // }
        // orbit_path.close();

        // if let Some(path) = orbit_path.finish() {
        //     paint.set_color_rgba8(127, 96, 64, 192);
        //     self.pixmap
        //         .stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        // }

        // let node_a = map_3d(focus + self.orbit.position_at(-self.orbit.arg_pe()));
        // let node_b = map_3d(focus + self.orbit.position_at(TAU / 2.0 - self.orbit.arg_pe()));
        // let mut node_path = PathBuilder::new();
        // node_path.move_to(node_a.x, node_a.y);
        // node_path.line_to(node_b.x, node_b.y);

        // paint.set_color_rgba8(127, 127, 127, 127);
        // self.pixmap.stroke_path(
        //     &node_path.finish().unwrap(),
        //     &paint,
        //     &stroke,
        //     Transform::identity(),
        //     None,
        // );

        // let peri = map_3d(focus + self.orbit.position_at(0.0));
        // let peri_path = PathBuilder::from_circle(peri.x, peri.y, 5.0).unwrap();
        // paint.set_color_rgba8(255, 0, 0, 192);
        // self.pixmap.fill_path(
        //     &peri_path,
        //     &paint,
        //     Default::default(),
        //     Transform::identity(),
        //     None,
        // );

        // let apo = map_3d(focus + self.orbit.position_at(TAU / 2.0));
        // let apo_path = PathBuilder::from_circle(apo.x, apo.y, 5.0).unwrap();
        // paint.set_color_rgba8(0, 0, 255, 192);
        // self.pixmap.fill_path(
        //     &apo_path,
        //     &paint,
        //     Default::default(),
        //     Transform::identity(),
        //     None,
        // );

        // if let Some(state) = &self.state {
        //     let position = focus + state.position;

        //     let base = map_3d(position);
        //     let tip = map_3d(position + state.velocity);
        //     let mut vel_path = PathBuilder::new();
        //     vel_path.move_to(base.x, base.y);
        //     vel_path.line_to(tip.x, tip.y);

        //     paint.set_color_rgba8(0, 255, 0, 255);
        //     self.pixmap.stroke_path(
        //         &vel_path.finish().unwrap(),
        //         &paint,
        //         &stroke,
        //         Transform::identity(),
        //         None,
        //     );

        //     let accel = state.position.cross(state.velocity).normalize();
        //     let base = map_3d(position);
        //     let tip = map_3d(position + accel);
        //     let mut accel_path = PathBuilder::new();
        //     accel_path.move_to(base.x, base.y);
        //     accel_path.line_to(tip.x, tip.y);

        //     paint.set_color_rgba8(0, 0, 255, 255);
        //     self.pixmap.stroke_path(
        //         &accel_path.finish().unwrap(),
        //         &paint,
        //         &stroke,
        //         Transform::identity(),
        //         None,
        //     );
        // }

        let center = map_3d(DVec3::ZERO);
        let radius = map_3d(DVec3::new(1.5e11, 0.0, 0.0));
        let mut radius_path = PathBuilder::new();
        radius_path.move_to(center.x, center.y);
        radius_path.line_to(radius.x, radius.y);

        paint.set_color_rgba8(127, 127, 127, 127);
        self.pixmap.stroke_path(
            &radius_path.finish().unwrap(),
            &paint,
            &stroke,
            Transform::identity(),
            None,
        );

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
