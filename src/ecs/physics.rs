use std::{collections::HashMap, f32::consts as f32, time::Duration};

use crate::{
    orbit::Orbit3D,
    time::{SimDuration, SimInstant},
};
use bevy::{
    core::FixedTimestep,
    hierarchy::Parent,
    math::{DVec3, Quat, Vec3},
    prelude::{
        Component, Entity, ParallelSystemDescriptorCoercion, Plugin, Query, Res, ResMut, SystemSet,
        Transform,
    },
};

pub const SIM_INTERVAL: Duration = Duration::from_millis(10);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(SimTimer::new()).add_system_set(
            SystemSet::new()
                .label("physics")
                .with_run_criteria(FixedTimestep::step(SIM_INTERVAL.as_secs_f64()))
                .with_system(sim_tick_system)
                .with_system(orbit_system.after(sim_tick_system))
                .with_system(angular_motion_system.after(sim_tick_system))
                .with_system(absolute_position_system.after(orbit_system)),
        );
    }
}

pub fn sim_tick_system(mut timer: ResMut<SimTimer>) {
    timer.now += SIM_INTERVAL.into();
}

pub struct SimTimer {
    now: SimInstant,
}

impl SimTimer {
    pub fn new() -> Self {
        Self {
            now: SimInstant::epoch(),
        }
    }

    pub fn now(&self) -> SimInstant {
        self.now
    }
}

#[derive(Default, Component)]
pub struct Position(pub DVec3);

#[derive(Default, Component)]
pub struct Velocity(pub DVec3);

#[derive(Component)]
pub struct OrbitalTrajectory {
    pub parent: Entity,
    pub orbit: Orbit3D,
}

#[derive(Component)]
pub enum AngularMotion {
    Standard(StandardAngularMotion),
    FixedRotation(FixedRotation),
}

/// Rotation rate in terms of the axis and angular velocity.
///
/// Designed for objects that can be torqued: Convenient to modify and easy to
/// incrementally update an existing orientation, but not as stable and probably
/// not as precise as `FixedDuration`.
pub struct StandardAngularMotion {
    pub rotation_axis: Vec3,
    // Radians per second
    pub rotation_rate: f32,
}

impl StandardAngularMotion {
    pub fn update_as_quat(&mut self, func: impl FnOnce(Quat) -> Quat) {
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
pub struct FixedRotation {
    pub epoch_orientation: Quat,
    pub rotation_axis: Vec3,
    pub rotation_period: SimDuration,
}

pub fn orbit_system(
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
pub struct TransformOrigin;

pub fn absolute_position_system(
    origin_query: Query<(&TransformOrigin, &Position)>,
    mut objects_query: Query<(Entity, &Position, &mut Transform)>,
    orbit_query: Query<&OrbitalTrajectory>,
    position_query: Query<&Position>,
    parent_query: Query<&Parent>,
) {
    // let origin = DVec3::ZERO;
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
        _ => -origin,
    };
    let absolute_position = position.0 + parent_position;
    absolute_positions.insert(entity, absolute_position);
    transform.translation = absolute_position.as_vec3();
    absolute_position
}

pub fn angular_motion_system(
    time: Res<SimTimer>,
    mut bodies: Query<(&AngularMotion, &mut Transform)>,
) {
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
