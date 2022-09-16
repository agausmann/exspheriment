use std::{collections::HashMap, f32::consts as f32, time::Duration};

use crate::{
    orbit::Orbit3D,
    time::{SimDuration, SimInstant},
};
use bevy::{
    hierarchy::Parent,
    math::{DVec3, Quat, Vec3},
    prelude::{
        Component, Entity, ParallelSystemDescriptorCoercion, Plugin, Query, Res, ResMut, SystemSet,
        Transform, With, Without,
    },
    time::FixedTimestep,
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
                .with_system(motion_system.after(sim_tick_system))
                .with_system(global_position_to_transform_system.after(motion_system))
                .with_system(angular_motion_system.after(sim_tick_system)),
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
pub struct GlobalPosition(pub DVec3);

#[derive(Component)]
pub struct RelativeMotion {
    pub relative_to: Entity,
    pub motion: Motion,
}

pub enum Motion {
    Fixed { position: DVec3 },
    Orbital(Orbit3D),
}

impl Motion {
    pub fn position(&self, time: SimInstant) -> DVec3 {
        match self {
            &Self::Fixed { position } => position,
            &Self::Orbital(orbit) => orbit.current_state(time).position,
        }
    }
}

pub struct FreeMotion {
    pub position: DVec3,
    pub velocity: DVec3,
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

// Marker component used to identify the origin body for computing global positions.
// Generally the player/camera should be the origin point to maximize the local precision.
#[derive(Component)]
pub struct WorldOrigin;

pub fn motion_system(
    timer: Res<SimTimer>,
    root_query: Query<(), (With<GlobalPosition>, Without<RelativeMotion>)>,
    origin_query: Query<(Entity, &WorldOrigin)>,
    relative_motion_query: Query<&RelativeMotion>,
    mut global_position_query: Query<(Entity, &mut GlobalPosition)>,
) {
    // Root query is not actually read, but is a sanity check to make sure that
    // there is exactly one root.
    let _ = root_query.single();

    let mut cached_global_positions = HashMap::new();

    // First, work upwards from origin entity to root.
    // This will provide base cases for determining positioning of other bodies,
    // which recurse over the parent until they find an ancestor whose position
    // has already been calculated. All other bodies share at least one ancestor
    // with the origin (the root body).
    let (origin_entity, _) = origin_query.single();
    let mut current_entity = origin_entity;
    let mut current_position = DVec3::ZERO;
    loop {
        cached_global_positions.insert(current_entity, current_position);

        if let Ok(relative_motion) = relative_motion_query.get(current_entity) {
            current_entity = relative_motion.relative_to;
            current_position -= relative_motion.motion.position(timer.now);
        } else {
            break;
        }
    }

    // Now iterate over all bodies and update global positions,
    // calculating and memoizing as needed:
    fn get_position(
        entity: Entity,
        now: SimInstant,
        relative_motion_query: &Query<&RelativeMotion>,
        cached_global_positions: &mut HashMap<Entity, DVec3>,
    ) -> DVec3 {
        if let Some(&position) = cached_global_positions.get(&entity) {
            return position;
        }
        let relative_motion = relative_motion_query
            .get(entity)
            .expect("internal error: non-root entity does not have RelativeMotion");
        let parent_position = get_position(
            relative_motion.relative_to,
            now,
            relative_motion_query,
            cached_global_positions,
        );
        let position = parent_position + relative_motion.motion.position(now);
        cached_global_positions.insert(entity, position);
        position
    }
    for (entity, mut global_position) in global_position_query.iter_mut() {
        global_position.0 = get_position(
            entity,
            timer.now,
            &relative_motion_query,
            &mut cached_global_positions,
        );
    }
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

pub fn global_position_to_transform_system(
    mut transform_query: Query<(Entity, &GlobalPosition, &mut Transform)>,
    parent_query: Query<&Parent>,
) {
    for (entity, global_position, mut transform) in transform_query.iter_mut() {
        debug_assert!(
            !parent_query.contains(entity),
            "transform conflict: entity has both GlobalPosition and Parent",
        );
        transform.translation = global_position.0.as_vec3();
    }
}
