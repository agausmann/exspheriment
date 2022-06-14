pub mod math;
pub mod orbit;
pub mod time;

use std::time::Duration;

use bevy::{
    app::App,
    core::FixedTimestep,
    prelude::{ResMut, SystemSet},
    DefaultPlugins,
};
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

fn main() {
    App::new()
        .insert_resource(SimTimer::new())
        .add_system_set(
            SystemSet::new()
                .with_system(sim_tick)
                .with_run_criteria(FixedTimestep::step(SIM_INTERVAL.as_secs_f64())),
        )
        .add_plugins(DefaultPlugins)
        .run()
}
