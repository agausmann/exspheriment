pub mod geometry;
pub mod hud;
pub mod math;
pub mod model;
pub mod orbit;
pub mod scene;
pub mod time;
pub mod viewport;
pub mod world;

use anyhow::Context;
use hud::Hud;
use pollster::block_on;
use scene::Scene;
use std::sync::Arc;
use viewport::Viewport;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use world::World;

pub type GraphicsContext = Arc<GraphicsContextInner>;

pub struct GraphicsContextInner {
    pub window: Window,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub render_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
}

impl GraphicsContextInner {
    async fn new(window: Window) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .context("failed to create adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        let render_format = surface
            .get_preferred_format(&adapter)
            .context("failed to select a render format")?;
        let depth_format = wgpu::TextureFormat::Depth32Float;

        Ok(Self {
            window,
            surface,
            device,
            queue,
            render_format,
            depth_format,
        })
    }

    fn reconfigure(&self) {
        self.surface.configure(
            &self.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.render_format,
                width: self.window.inner_size().width,
                height: self.window.inner_size().height,
                present_mode: wgpu::PresentMode::Fifo,
            },
        );
    }
}

struct App {
    gfx: GraphicsContext,
    viewport: Viewport,
    world: World,
    scene: Scene,
    hud: Hud,
}

impl App {
    async fn new(window: Window) -> anyhow::Result<Self> {
        let gfx = Arc::new(GraphicsContextInner::new(window).await?);
        gfx.reconfigure();

        let viewport = Viewport::new(&gfx);
        let world = World::new();
        let scene = Scene::new(&gfx, &viewport);
        let hud = Hud::new(&gfx);

        Ok(Self {
            gfx,
            viewport,
            world,
            scene,
            hud,
        })
    }

    fn update(&mut self) {
        self.viewport.update();
        // self.scene.update(&self.viewport);
        // self.hud.orbit = self.scene.orbit;
        // self.hud.state = self.scene.state;

        self.world.update();
        for (id, tag) in self.world.body_tags.iter().enumerate() {
            self.scene.instances[id].model = self.world.body(tag).model_matrix().to_cols_array_2d();
            self.scene.instances[id].albedo = [0.3, 0.6, 0.9];
        }
    }

    fn redraw(&mut self) -> anyhow::Result<()> {
        let frame = loop {
            match self.gfx.surface.get_current_texture() {
                Ok(frame) => {
                    if frame.suboptimal {
                        self.gfx.reconfigure();
                    } else {
                        break frame;
                    }
                }
                Err(wgpu::SurfaceError::Lost) => {
                    self.gfx.reconfigure();
                }
                Err(wgpu::SurfaceError::Timeout) | Err(wgpu::SurfaceError::Outdated) => {
                    return Ok(());
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        };

        let size = self.gfx.window.inner_size();

        let depth_texture = self.gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: wgpu::Extent3d {
                width: size.width,
                height: size.height,
                ..Default::default()
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.gfx.depth_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });

        let frame_view = frame.texture.create_view(&Default::default());
        let depth_view = depth_texture.create_view(&Default::default());
        let mut encoder = self.gfx.device.create_command_encoder(&Default::default());
        self.scene
            .draw(&mut encoder, &frame_view, &depth_view, &self.viewport);
        self.hud.draw(&mut encoder, &frame_view, &self.viewport);

        self.gfx.queue.submit([encoder.finish()]);
        frame.present();

        Ok(())
    }

    fn window_resized(&mut self) {
        self.gfx.reconfigure();
        self.hud.resized();
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(1280, 720))
        .with_title("Exspherement")
        .build(&event_loop)?;

    let mut app = block_on(App::new(window))?;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(..) => {
            app.update();
            app.redraw().unwrap();
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
                app.window_resized();
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            app.gfx.window.request_redraw();
        }
        _ => {}
    })
}
