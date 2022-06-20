use std::f64::consts as f64;

use bevy::{
    core::FixedTimestep,
    input::{mouse::MouseMotion, Input},
    math::DVec3,
    prelude::{
        Component, EventReader, KeyCode, MouseButton, Plugin, Query, Res, ResMut, SystemSet,
        Transform,
    },
    window::{WindowFocused, Windows},
};

use crate::orbit::State3D;

use super::physics::{Motion, RelativeMotion, SimTimer, SIM_INTERVAL};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(FocusState::default()).add_system_set(
            SystemSet::new()
                .label("input")
                .with_run_criteria(FixedTimestep::step(SIM_INTERVAL.as_secs_f64()))
                .with_system(controller_system)
                .with_system(cursor_grab_system),
        );
    }
}

#[derive(Debug, Component)]
pub struct Controller {
    pub pitch: f64,
    pub yaw: f64,
}

/// Mouse look sensitivity (degrees per dot)
const SENSITIVITY: f64 = 0.2;

/// Move speed for fixed-position camera controller (meters per second)
const MOVE_SPEED: f64 = 3.0;

/// Acceleration factor for orbital motion controller (units TBD)
const ORBIT_ACCEL: f64 = 0.1;

pub struct FocusState {
    window_focused: bool,
    grabbed: bool,
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            window_focused: true,
            grabbed: false,
        }
    }
}

pub fn controller_system(
    mut camera: Query<(&mut Controller, &mut RelativeMotion, &mut Transform)>,
    time: Res<SimTimer>,
    focus: Res<FocusState>,
    keys: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
) {
    let (mut controller, mut relative_motion, mut transform) = camera.single_mut();

    for event in mouse.iter() {
        let event_delta = event.delta.as_dvec2();
        if focus.grabbed && focus.window_focused {
            controller.yaw += SENSITIVITY * f64::TAU / 360.0 * -event_delta.x;
            controller.yaw %= f64::TAU;

            controller.pitch += SENSITIVITY * f64::TAU / 360.0 * -event_delta.y;
            controller.pitch = controller
                .pitch
                .clamp(1.0e-3 - f64::TAU / 4.0, f64::TAU / 4.0 - 1.0e-3);
        }
    }

    let relative_forward = DVec3::new(
        controller.yaw.cos() * controller.pitch.cos(),
        controller.yaw.sin() * controller.pitch.cos(),
        controller.pitch.sin(),
    );
    let aligned_up = DVec3::Z;
    let right = relative_forward.cross(aligned_up).normalize();
    let aligned_forward = aligned_up.cross(right);
    let relative_up = right.cross(relative_forward);

    let (effective_forward, effective_up) = match relative_motion.motion {
        // Axis-aligned to make freecam a bit nicer
        Motion::Fixed { .. } => (aligned_forward, aligned_up),
        // Relative to view direction to make thrust nicer.
        Motion::Orbital(..) => (relative_forward, relative_up),
    };

    if focus.grabbed && focus.window_focused {
        let mut delta = DVec3::ZERO;
        if keys.pressed(KeyCode::W) {
            delta += effective_forward;
        }
        if keys.pressed(KeyCode::S) {
            delta -= effective_forward;
        }
        if keys.pressed(KeyCode::A) {
            delta -= right;
        }
        if keys.pressed(KeyCode::D) {
            delta += right;
        }
        if keys.pressed(KeyCode::Space) {
            delta += effective_up;
        }
        if keys.pressed(KeyCode::LShift) {
            delta -= effective_up;
        }
        match &mut relative_motion.motion {
            &mut Motion::Fixed { ref mut position } => {
                *position += delta * MOVE_SPEED * SIM_INTERVAL.as_secs_f64();
            }
            &mut Motion::Orbital(ref mut orbit) => {
                if delta != DVec3::ZERO {
                    let dv = delta * ORBIT_ACCEL * SIM_INTERVAL.as_secs_f64();
                    let current_state = orbit.current_state(time.now());
                    let new_state = State3D {
                        velocity: current_state.velocity + dv,
                        ..current_state
                    };
                    orbit.update_from_current_state(&new_state);
                }
            }
        }
    }

    let target = transform.translation + relative_forward.as_vec3();
    transform.look_at(target, aligned_up.as_vec3());
}

fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    mut focus_events: EventReader<WindowFocused>,
    mut focus_state: ResMut<FocusState>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();

    for event in focus_events.iter() {
        if event.id == window.id() {
            focus_state.window_focused = event.focused
        }
    }

    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
        focus_state.grabbed = true;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
        focus_state.grabbed = false;
    }
}
