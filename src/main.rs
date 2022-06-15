pub mod geometry;
pub mod math;
pub mod orbit;
pub mod time;

use std::time::Duration;

use bevy::{
    app::App,
    core::FixedTimestep,
    math::DVec3,
    prelude::{Component, ParallelSystemDescriptorCoercion, Query, Res, ResMut, SystemSet},
    DefaultPlugins,
};
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

fn main() {
    App::new()
        .insert_resource(SimTimer::new())
        .add_system_set(
            SystemSet::new()
                .with_system(sim_tick)
                .with_system(update_orbits.after(sim_tick))
                .with_run_criteria(FixedTimestep::step(SIM_INTERVAL.as_secs_f64())),
        )
        .add_plugins(DefaultPlugins)
        .run()
}
