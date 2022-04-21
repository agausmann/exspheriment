use std::mem::replace;

use glam::Vec3;
use winit::event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent};

pub struct Controls {
    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    yaw: f64,
    pitch: f64,
}

impl Controls {
    pub fn new() -> Self {
        Self {
            forward: false,
            back: false,
            left: false,
            right: false,
            up: false,
            down: false,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn event<T>(&mut self, event: &Event<T>) {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                    Some(VirtualKeyCode::W) => self.forward = input.state.is_pressed(),
                    Some(VirtualKeyCode::S) => self.back = input.state.is_pressed(),
                    Some(VirtualKeyCode::A) => self.left = input.state.is_pressed(),
                    Some(VirtualKeyCode::D) => self.right = input.state.is_pressed(),
                    Some(VirtualKeyCode::Space) => self.up = input.state.is_pressed(),
                    Some(VirtualKeyCode::LShift) => self.down = input.state.is_pressed(),
                    _ => {}
                },
                _ => {}
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta, .. } => {
                    let (x, y) = delta;
                    self.yaw -= x;
                    self.pitch -= y;
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn net_movement(&self) -> Vec3 {
        Vec3::new(
            self.right as u8 as f32 - self.left as u8 as f32,
            self.forward as u8 as f32 - self.back as u8 as f32,
            self.up as u8 as f32 - self.down as u8 as f32,
        )
    }

    pub fn take_yaw(&mut self) -> f64 {
        replace(&mut self.yaw, 0.0)
    }

    pub fn take_pitch(&mut self) -> f64 {
        replace(&mut self.pitch, 0.0)
    }
}

trait ElementStateExt {
    fn is_pressed(&self) -> bool;
}

impl ElementStateExt for ElementState {
    fn is_pressed(&self) -> bool {
        matches!(self, ElementState::Pressed)
    }
}
