pub mod geometry;
pub mod math;
pub mod orbit;
pub mod time;

use std::time::Duration;

use bevy::{
    app::App,
    core::FixedTimestep,
    math::{DVec3, Vec3},
    pbr::{DirectionalLightBundle, MaterialMeshBundle, StandardMaterial},
    prelude::{
        Assets, Color, Commands, Component, Mesh, ParallelSystemDescriptorCoercion,
        PerspectiveCameraBundle, Query, Res, ResMut, SystemSet, Transform, Visibility,
    },
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
        subdivisions: 1,
        method: SubdivisionMethod::Slerp,
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
    commands.spawn().insert_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(1.0, 3.0, 2.0))
            .looking_at(Vec3::ZERO, Vec3::Z),
        ..Default::default()
    });
    commands
        .spawn()
        .insert_bundle(DirectionalLightBundle::default());
}

fn main() {
    App::new()
        .insert_resource(SimTimer::new())
        .add_startup_system(setup)
        .add_system_set(
            SystemSet::new()
                .with_system(sim_tick)
                .with_system(update_orbits.after(sim_tick))
                .with_run_criteria(FixedTimestep::step(SIM_INTERVAL.as_secs_f64())),
        )
        .add_plugins(DefaultPlugins)
        .run()
}
