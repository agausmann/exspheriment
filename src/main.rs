pub mod geometry;
pub mod math;
pub mod orbit;
pub mod time;

use std::{f32::consts::TAU, time::Duration};

use bevy::{
    app::App,
    core::FixedTimestep,
    input::{mouse::MouseMotion, Input},
    math::{DVec3, Quat, Vec3},
    pbr::{DirectionalLightBundle, MaterialMeshBundle, StandardMaterial},
    prelude::{
        Assets, Color, Commands, Component, EventReader, KeyCode, Mesh, MouseButton,
        ParallelSystemDescriptorCoercion, PerspectiveCameraBundle, Query, Res, ResMut, SystemSet,
        Transform, Visibility,
    },
    window::Windows,
    DefaultPlugins,
};
use geometry::{Geodesic, SubdivisionMethod};
use orbit::Orbit3D;
use time::SimInstant;

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

fn sim_tick(mut timer: ResMut<SimTimer>) {
    timer.now += SIM_INTERVAL.into();
}

#[derive(Component)]
struct Orbit(Orbit3D);

#[derive(Component)]
struct Position(DVec3);

#[derive(Component)]
struct Velocity(DVec3);

fn update_orbits(
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let geodesic = meshes.add(Mesh::from(Geodesic {
        subdivisions: 4,
        method: SubdivisionMethod::Lerp,
    }));
    let earth_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.0, 0.1, 0.3),
        ..Default::default()
    });

    commands.spawn().insert_bundle(MaterialMeshBundle {
        mesh: geodesic,
        material: earth_material,
        transform: Transform::identity(),
        global_transform: Default::default(),
        visibility: Visibility { is_visible: true },
        computed_visibility: Default::default(),
    });
    commands
        .spawn()
        .insert_bundle(PerspectiveCameraBundle {
            transform: Transform::from_translation(Vec3::new(1.0, 3.0, 2.0))
                .looking_at(Vec3::ZERO, Vec3::Z),
            ..Default::default()
        })
        .insert(Controller {
            position: Vec3::new(0.0, -2.0, 0.0),
            pitch: 0.0,
            yaw: 0.0,
        });
    commands
        .spawn()
        .insert_bundle(DirectionalLightBundle::default());
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

fn update_camera(
    mut camera: Query<(&mut Controller, &mut Transform)>,
    keys: Res<Input<KeyCode>>,
    mut mouse: EventReader<MouseMotion>,
) {
    let (mut controller, mut transform) = camera.single_mut();

    for event in mouse.iter() {
        controller.yaw += SENSITIVITY * TAU / 360.0 * -event.delta.x;
        controller.yaw %= TAU;

        controller.pitch += SENSITIVITY * TAU / 360.0 * -event.delta.y;
        controller.pitch = controller.pitch.clamp(-TAU / 4.0, TAU / 4.0);
    }

    let forward = Vec3::new(
        controller.yaw.cos() * controller.pitch.cos(),
        controller.yaw.sin() * controller.pitch.cos(),
        controller.pitch.sin(),
    );
    let up = Vec3::Z;
    let right = forward.cross(up).normalize();
    let flat_forward = up.cross(right);

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

    *transform = Transform {
        translation: controller.position,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }
    .looking_at(controller.position + forward, up)
}

fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }
}

fn main() {
    App::new()
        .insert_resource(SimTimer::new())
        .add_startup_system(setup)
        .add_system(cursor_grab_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(SIM_INTERVAL.as_secs_f64()))
                .with_system(sim_tick)
                .with_system(update_orbits.after(sim_tick))
                .with_system(update_camera),
        )
        .add_plugins(DefaultPlugins)
        .run()
}
