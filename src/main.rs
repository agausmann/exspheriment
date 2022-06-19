pub mod ecs;
pub mod geometry;
pub mod math;
pub mod orbit;
pub mod time;

use std::{f32::consts as f32, f64::consts as f64};

use bevy::{
    app::App,
    core_pipeline::ClearColor,
    math::{DVec3, Quat, Vec3},
    pbr::{AmbientLight, DirectionalLightBundle, MaterialMeshBundle, StandardMaterial},
    prelude::{Assets, Color, Commands, Mesh, PerspectiveCameraBundle, ResMut, Transform},
    DefaultPlugins,
};
use ecs::{
    input::Controller,
    physics::{
        AngularMotion, FixedRotation, OrbitalTrajectory, Position, StandardAngularMotion,
        TransformOrigin, Velocity,
    },
};
use geometry::{Geodesic, SubdivisionMethod};
use orbit::{Orbit2D, Orbit3D};
use time::{SimDuration, SimInstant};

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

    let body_2 = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(Geodesic {
                subdivisions: 2,
                method: SubdivisionMethod::Lerp,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.8, 0.8, 0.8),
                ..Default::default()
            }),
            transform: Transform {
                scale: Vec3::splat(0.3),
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

    let _body_3 = commands
        .spawn()
        .insert_bundle(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(Geodesic {
                subdivisions: 1,
                method: SubdivisionMethod::Lerp,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.5, 0.0, 0.0),
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
            parent: body_2,
            orbit: Orbit3D::new(
                Orbit2D::from_apsides(1.0, 1.0, SimInstant::epoch(), 0.1),
                0.0,
                f64::TAU / 8.0,
                0.0,
            ),
        })
        .insert(AngularMotion::Standard(StandardAngularMotion {
            rotation_axis: Vec3::X,
            rotation_rate: 0.0,
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ecs::input::InputPlugin)
        .add_plugin(ecs::physics::PhysicsPlugin)
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.02,
        })
        .add_startup_system(setup_system)
        .run();
}
