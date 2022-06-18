pub mod geometry;
pub mod math;
pub mod orbit;
pub mod time;

use std::{collections::HashMap, f32::consts as f32, f64::consts as f64, time::Duration};

use bevy::{
    app::App,
    core::FixedTimestep,
    core_pipeline::ClearColor,
    hierarchy::Parent,
    input::{mouse::MouseMotion, Input},
    math::{DVec3, Quat, Vec3},
    pbr::{AmbientLight, DirectionalLightBundle, MaterialMeshBundle, StandardMaterial},
    prelude::{
        Assets, Color, Commands, Component, Entity, EventReader, KeyCode, Mesh, MouseButton,
        ParallelSystemDescriptorCoercion, PerspectiveCameraBundle, Query, Res, ResMut, SystemSet,
        Transform,
    },
    window::{WindowFocused, Windows},
    DefaultPlugins,
};
use geometry::{Geodesic, SubdivisionMethod};
use orbit::{Orbit2D, Orbit3D};
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
struct OrbitalTrajectory {
    parent: Entity,
    orbit: Orbit3D,
}

#[derive(Component)]
enum AngularMotion {
    Standard(StandardAngularMotion),
    FixedRotation(FixedRotation),
}

/// Rotation rate in terms of the axis and angular velocity.
///
/// Designed for objects that can be torqued: Convenient to modify and easy to
/// incrementally update an existing orientation, but not as stable and probably
/// not as precise as `FixedDuration`.
struct StandardAngularMotion {
    rotation_axis: Vec3,
    // Radians per second
    rotation_rate: f32,
}

impl StandardAngularMotion {
    fn update_as_quat(&mut self, func: impl FnOnce(Quat) -> Quat) {
        let (new_axis, new_rate) = func(Quat::from_axis_angle(
            self.rotation_axis,
            self.rotation_rate,
        ))
        .to_axis_angle();
        self.rotation_axis = new_axis;
        self.rotation_rate = new_rate;
    }
}

/// Orientation and angular motion in terms of an initial orientation, axis of
/// rotation, and period.
///
/// A convenient representation for fixed bodies (e.g. planets), with a precise
/// rotation rate stored as an exact `SimDuration`, but not designed to be
/// updated.
struct FixedRotation {
    epoch_orientation: Quat,
    rotation_axis: Vec3,
    rotation_period: SimDuration,
}

#[derive(Default, Component)]
struct Position(DVec3);

#[derive(Default, Component)]
struct Velocity(DVec3);

fn orbit_system(
    timer: Res<SimTimer>,
    mut orbits: Query<(
        &OrbitalTrajectory,
        Option<&mut Position>,
        Option<&mut Velocity>,
    )>,
) {
    for (orbit, position, velocity) in orbits.iter_mut() {
        let state = orbit.orbit.current_state(timer.now);
        if let Some(mut position) = position {
            position.0 = state.position;
        }
        if let Some(mut velocity) = velocity {
            velocity.0 = state.velocity;
        }
    }
}

// Marker component used to identify the origin when computing transforms.
#[derive(Component)]
struct TransformOrigin;

fn absolute_position_system(
    origin_query: Query<(&TransformOrigin, &Position)>,
    mut objects_query: Query<(Entity, &Position, &mut Transform)>,
    orbit_query: Query<&OrbitalTrajectory>,
    position_query: Query<&Position>,
    parent_query: Query<&Parent>,
) {
    let origin: DVec3 = match origin_query.get_single() {
        Ok((_, position)) => position.0,
        _ => return,
    };
    let mut visited = HashMap::new();

    for (entity, position, mut transform) in objects_query.iter_mut() {
        absolute_position_inner(
            entity,
            position,
            &mut *transform,
            origin,
            &mut visited,
            &orbit_query,
            &position_query,
            &parent_query,
        );
    }
}

fn absolute_position_inner(
    entity: Entity,
    position: &Position,
    transform: &mut Transform,
    origin: DVec3,
    absolute_positions: &mut HashMap<Entity, DVec3>,
    orbit_query: &Query<&OrbitalTrajectory>,
    position_query: &Query<&Position>,
    parent_query: &Query<&Parent>,
) -> DVec3 {
    if let Some(&position) = absolute_positions.get(&entity) {
        return position;
    }
    debug_assert!(
        !parent_query.contains(entity),
        "conflicting global transforms for entity: has both a Parent and a Position"
    );
    let parent_position = match orbit_query.get(entity) {
        Ok(orbit) => {
            let relative_position = position_query
                .get(orbit.parent)
                .expect("orbit parent does not have a Position");
            absolute_position_inner(
                orbit.parent,
                relative_position,
                transform,
                origin,
                absolute_positions,
                orbit_query,
                position_query,
                parent_query,
            )
        }
        _ => DVec3::ZERO,
    };
    let absolute_position = position.0 + parent_position - origin;
    absolute_positions.insert(entity, absolute_position);
    transform.translation = absolute_position.as_vec3();
    absolute_position
}

fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let body_1 = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(Geodesic {
                subdivisions: 8,
                method: SubdivisionMethod::Lerp,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.0, 0.1, 0.3),
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(Position(DVec3::ZERO))
        .insert(AngularMotion::FixedRotation(FixedRotation {
            rotation_axis: Vec3::Z,
            epoch_orientation: Quat::IDENTITY,
            rotation_period: SimDuration::from_secs_f64(60.0),
        }))
        .id();

    let _body_2 = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(Geodesic {
                subdivisions: 1,
                method: SubdivisionMethod::Lerp,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.8, 0.8),
                ..Default::default()
            }),
            transform: Transform {
                scale: Vec3::splat(0.1),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Position::default())
        .insert(Velocity::default())
        .insert(OrbitalTrajectory {
            parent: body_1,
            orbit: Orbit3D::new(
                Orbit2D::from_apsides(5.0, 5.0, SimInstant::epoch(), 1.0),
                0.0,
                0.0,
                0.0,
            ),
        })
        .insert(AngularMotion::FixedRotation(FixedRotation {
            epoch_orientation: Quat::IDENTITY,
            rotation_axis: Vec3::Z,
            rotation_period: SimDuration::from_secs_f64(120.0),
        }))
        .id();

    commands
        .spawn()
        .insert_bundle(PerspectiveCameraBundle::default())
        .insert(Controller {
            pitch: 0.0,
            yaw: f64::TAU / 4.0,
        })
        .insert(Position(DVec3::new(0.0, -4.0, 0.0)))
        .insert(TransformOrigin);
    commands.spawn().insert_bundle(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_y(f32::TAU / 8.0)),
        ..Default::default()
    });
}

#[derive(Debug, Component)]
struct Controller {
    pitch: f64,
    yaw: f64,
}

/// Mouse look sensitivity (degrees per dot)
const SENSITIVITY: f64 = 0.2;

/// Move speed (meters per second)
const MOVE_SPEED: f64 = 3.0;

struct FocusState {
    window_focused: bool,
    grabbed: bool,
}

fn controller_system(
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

    *transform = Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    }
    .looking_at(forward.as_vec3(), up.as_vec3())
}

fn angular_motion_system(time: Res<SimTimer>, mut bodies: Query<(&AngularMotion, &mut Transform)>) {
    for (motion, mut transform) in bodies.iter_mut() {
        match motion {
            AngularMotion::FixedRotation(motion) => {
                let time_in_current_day = (time.now - SimInstant::epoch()) % motion.rotation_period;
                let current_angle = f32::TAU
                    * (time_in_current_day.as_secs_f32() / motion.rotation_period.as_secs_f32());
                transform.rotation = motion.epoch_orientation
                    * Quat::from_axis_angle(motion.rotation_axis, current_angle)
            }
            AngularMotion::Standard(motion) => {
                let angle_delta = motion.rotation_rate * SIM_INTERVAL.as_secs_f32();
                let quat = Quat::from_axis_angle(motion.rotation_axis, angle_delta);
                transform.rotation = (quat * transform.rotation).normalize();
            }
        }
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
                .with_system(angular_motion_system.after(sim_tick_system))
                .with_system(absolute_position_system.after(orbit_system))
                .with_system(controller_system),
        )
        .run();
}
