pub mod geometry;
pub mod math;
pub mod orbit;
pub mod time;

use std::{f32::consts::TAU, time::Duration};

use bevy::{
    app::App,
    core::FixedTimestep,
    core_pipeline::ClearColor,
    input::{mouse::MouseMotion, Input},
    math::{DVec3, Quat, Vec3},
    pbr::{AmbientLight, DirectionalLightBundle, MaterialMeshBundle, StandardMaterial},
    prelude::{
        Assets, Color, Commands, Component, EventReader, KeyCode, Mesh, MouseButton,
        ParallelSystemDescriptorCoercion, PerspectiveCameraBundle, Query, Res, ResMut, SystemSet,
        Transform, Visibility,
    },
    window::{WindowFocused, Windows},
    DefaultPlugins,
};
use geometry::{Geodesic, SubdivisionMethod};
use orbit::Orbit3D;
use time::{SimDuration, SimInstant};

const SIM_INTERVAL: Duration = Duration::from_millis(10);

struct SimTimer {
    now: SimInstant,
}

impl SimTimer {
    fn new() -> Self {
        Self {
            now: SimInstant::epoch(),
        }
    }
}

fn sim_tick_system(mut timer: ResMut<SimTimer>) {
    timer.now += SIM_INTERVAL.into();
}

#[derive(Component)]
struct Orbit(Orbit3D);

#[derive(Component)]
struct Position(DVec3);

#[derive(Component)]
struct Velocity(DVec3);

#[derive(Component)]
struct FixedRotation {
    axis: Vec3,
    tilt: Quat,
    period: SimDuration,
    start: SimInstant,
}

fn orbit_system(
    timer: Res<SimTimer>,
    mut orbits: Query<(&Orbit, Option<&mut Position>, Option<&mut Velocity>)>,
) {
    for (orbit, position, velocity) in orbits.iter_mut() {
        let state = orbit.0.current_state(timer.now);
        if let Some(mut position) = position {
            position.0 = state.position;
        }
        if let Some(mut velocity) = velocity {
            velocity.0 = state.velocity;
        }
    }
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let geodesic = meshes.add(Mesh::from(Geodesic {
        subdivisions: 8,
        method: SubdivisionMethod::Lerp,
    }));
    let earth_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.0, 0.1, 0.3),
        ..Default::default()
    });

    commands
        .spawn()
        .insert_bundle(MaterialMeshBundle {
            mesh: geodesic,
            material: earth_material,
            transform: Transform::identity(),
            global_transform: Default::default(),
            visibility: Visibility { is_visible: true },
            computed_visibility: Default::default(),
        })
        .insert(FixedRotation {
            axis: Vec3::Z,
            tilt: Quat::IDENTITY,
            period: SimDuration::from_secs_f64(60.0),
            start: SimInstant::epoch(),
        });
    commands
        .spawn()
        .insert_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(1.0, 3.0, 2.0))
                .looking_at(Vec3::ZERO, Vec3::Z),
            ..Default::default()
        })
        .insert(Controller {
            position: Vec3::new(0.0, -4.0, 0.0),
            pitch: 0.0,
            yaw: TAU / 4.0,
        });
    commands.spawn().insert_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_y(TAU / 8.0)),
        ..Default::default()
    });
}

#[derive(Debug, Component)]
struct Controller {
    position: Vec3,
    pitch: f32,
    yaw: f32,
}

/// Mouse look sensitivity (degrees per dot)
const SENSITIVITY: f32 = 0.2;

/// Move speed (meters per second)
const MOVE_SPEED: f32 = 3.0;

struct FocusState {
    window_focused: bool,
    grabbed: bool,
}

fn controller_system(
    mut camera: Query<(&mut Controller, &mut Transform)>,
    focus: Res<FocusState>,
    keys: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
) {
    let (mut controller, mut transform) = camera.single_mut();

    for event in mouse.iter() {
        if focus.grabbed && focus.window_focused {
            controller.yaw += SENSITIVITY * TAU / 360.0 * -event.delta.x;
            controller.yaw %= TAU;

            controller.pitch += SENSITIVITY * TAU / 360.0 * -event.delta.y;
            controller.pitch = controller
                .pitch
                .clamp(1.0e-3 - TAU / 4.0, TAU / 4.0 - 1.0e-3);
        }
    }

    let forward = Vec3::new(
        controller.yaw.cos() * controller.pitch.cos(),
        controller.yaw.sin() * controller.pitch.cos(),
        controller.pitch.sin(),
    );
    let up = Vec3::Z;
    let right = forward.cross(up).normalize();
    let flat_forward = up.cross(right);

    if focus.grabbed && focus.window_focused {
        let mut delta = Vec3::ZERO;
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
        controller.position += delta * MOVE_SPEED * SIM_INTERVAL.as_secs_f32();
    }

    *transform = Transform {
        translation: controller.position,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }
    .looking_at(controller.position + forward, up)
}

fn fixed_rotation_system(time: Res<SimTimer>, mut bodies: Query<(&FixedRotation, &mut Transform)>) {
    for (rotation, mut transform) in bodies.iter_mut() {
        let time_in_current_day = (time.now - rotation.start) % rotation.period;
        let current_angle =
            TAU * (time_in_current_day.as_secs_f32() / rotation.period.as_secs_f32());
        transform.rotation = rotation.tilt * Quat::from_axis_angle(rotation.axis, current_angle)
    }
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(SimTimer::new())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.02,
        })
        .insert_resource(FocusState {
            grabbed: false,
            window_focused: false,
        })
        .add_startup_system(setup_system)
        .add_system(cursor_grab_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(SIM_INTERVAL.as_secs_f64()))
                .with_system(sim_tick_system)
                .with_system(orbit_system.after(sim_tick_system))
                .with_system(fixed_rotation_system.after(sim_tick_system))
                .with_system(controller_system),
        )
        .run();
}
