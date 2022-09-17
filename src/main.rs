pub mod ecs;
pub mod geometry;
pub mod math;
pub mod orbit;
pub mod time;

use std::{f32::consts as f32, f64::consts as f64};

use bevy::{
    app::App,
    core_pipeline::clear_color::ClearColor,
    math::{DVec3, Quat, Vec3},
    pbr::{
        AmbientLight, DirectionalLightBundle, MaterialMeshBundle, NotShadowCaster, StandardMaterial,
    },
    prelude::{
        Assets, Camera3dBundle, Color, Commands, Mesh, PointLight, PointLightBundle, ResMut,
        Transform,
    },
    DefaultPlugins,
};
use ecs::{
    input::Controller,
    physics::{
        AngularMotion, FixedRotation, GlobalPosition, Motion, RelativeMotion,
        StandardAngularMotion, WorldOrigin,
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
                // base_color: Color::rgb(0.0, 0.1, 0.3),
                emissive: Color::rgb(1.0, 0.2, 0.0),
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(GlobalPosition(DVec3::ZERO))
        .insert(AngularMotion::FixedRotation(FixedRotation {
            rotation_axis: Vec3::Z,
            epoch_orientation: Quat::IDENTITY,
            rotation_period: SimDuration::from_secs_f64(60.0),
        }))
        .insert_bundle(PointLightBundle {
            point_light: PointLight {
                color: Color::rgb(1.0, 0.2, 0.0),
                intensity: 1000.0,
                range: 100.0,
                radius: 1.0,
                shadows_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(NotShadowCaster)
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
        .insert(GlobalPosition::default())
        .insert(RelativeMotion {
            relative_to: body_1,
            motion: Motion::Orbital(Orbit3D::new(
                Orbit2D::from_apsides(5.0, 5.0, SimInstant::epoch(), 1.0),
                0.0,
                0.0,
                0.0,
            )),
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
        .insert(GlobalPosition::default())
        .insert(RelativeMotion {
            relative_to: body_2,
            motion: Motion::Orbital(Orbit3D::new(
                Orbit2D::from_apsides(1.0, 1.0, SimInstant::epoch(), 0.1),
                0.0,
                f64::TAU / 8.0,
                0.0,
            )),
        })
        .insert(AngularMotion::Standard(StandardAngularMotion {
            rotation_axis: Vec3::X,
            rotation_rate: 0.0,
        }))
        .id();

    commands
        .spawn()
        .insert_bundle(Camera3dBundle::default())
        .insert(Controller {
            pitch: 0.0,
            yaw: f64::TAU / 4.0,
        })
        .insert(GlobalPosition::default())
        // .insert(RelativeMotion {
        //     relative_to: body_2,
        //     motion: Motion::Orbital(Orbit3D::new(
        //         Orbit2D::from_apsides(0.8, 0.8, SimInstant::epoch(), 0.1),
        //         0.0,
        //         0.0,
        //         0.0,
        //     )),
        // })
        .insert(RelativeMotion {
            relative_to: body_1,
            motion: Motion::Fixed {
                position: DVec3::new(2.0, 0.0, 0.0),
            },
        })
        .insert(WorldOrigin);
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
