use std::f32::consts::TAU;

use bevy::{
    prelude::Mesh,
    render::mesh::{Indices, PrimitiveTopology},
};
use once_cell::sync::Lazy;

fn vertices() -> Vec<[f32; 3]> {
    let mut vertices = Vec::with_capacity(12);

    vertices.push([0.0, 0.0, 1.0]);

    let phi = (0.5_f32).atan();
    for i in 0..5 {
        let theta = (i as f32) / 5.0 * TAU;
        vertices.push([theta.cos() * phi.cos(), theta.sin() * phi.cos(), phi.sin()]);
    }

    let phi = -phi;
    for i in 0..5 {
        let theta = (i as f32 + 0.5) / 5.0 * TAU;
        vertices.push([theta.cos() * phi.cos(), theta.sin() * phi.cos(), phi.sin()]);
    }

    vertices.push([0.0, 0.0, -1.0]);

    vertices
}

pub static VERTICES: Lazy<Vec<[f32; 3]>> = Lazy::new(vertices);

#[rustfmt::skip]
pub const INDICES: [u16; 60] = [
    1, 0, 2,
    2, 0, 3,
    3, 0, 4,
    4, 0, 5,
    5, 0, 1,

    6, 7, 11,
    7, 8, 11,
    8, 9, 11,
    9, 10, 11,
    10, 6, 11,

    1, 2, 6,
    2, 3, 7,
    3, 4, 8,
    4, 5, 9,
    5, 1, 10,
    
    6, 2, 7,
    7, 3, 8,
    8, 4, 9,
    9, 5, 10,
    10, 1, 6,
];

pub struct Icosahedron;

impl From<Icosahedron> for Mesh {
    fn from(_icos: Icosahedron) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VERTICES.clone());
        mesh.set_indices(Some(Indices::U16(INDICES.into())));
        mesh
    }
}
