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

use super::physics::{Position, SIM_INTERVAL};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(FocusState {
            grabbed: false,
            window_focused: false,
        })
        .add_system_set(
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

/// Move speed (meters per second)
const MOVE_SPEED: f64 = 3.0;

pub struct FocusState {
    window_focused: bool,
    grabbed: bool,
}

pub fn controller_system(
    mut camera: Query<(&mut Controller, &mut Position, &mut Transform)>,
    focus: Res<FocusState>,
    keys: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
) {
    let (mut controller, mut position, mut transform) = camera.single_mut();

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

    let forward = DVec3::new(
        controller.yaw.cos() * controller.pitch.cos(),
        controller.yaw.sin() * controller.pitch.cos(),
        controller.pitch.sin(),
    );
    let up = DVec3::Z;
    let right = forward.cross(up).normalize();
    let flat_forward = up.cross(right);

    if focus.grabbed && focus.window_focused {
        let mut delta = DVec3::ZERO;
        if keys.pressed(KeyCode::W) {
            delta += flat_forward;
        }
        if keys.pressed(KeyCode::S) {
            delta -= flat_forward;
        }
        if keys.pressed(KeyCode::A) {
            delta -= right;
        }
        if keys.pressed(KeyCode::D) {
            delta += right;
        }
        if keys.pressed(KeyCode::Space) {
            delta += up;
        }
        if keys.pressed(KeyCode::LShift) {
            delta -= up;
        }
        position.0 += delta * MOVE_SPEED * SIM_INTERVAL.as_secs_f64();
    }

    let target = transform.translation + forward.as_vec3();
    transform.look_at(target, up.as_vec3());
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
