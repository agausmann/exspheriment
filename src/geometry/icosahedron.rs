use std::f32::consts::TAU;

use glam::Vec3;
use once_cell::sync::Lazy;

use crate::{model::Model, GraphicsContext};

fn vertices() -> [Vec3; 12] {
    let mut vertices: [Vec3; 12] = Default::default();

    vertices[0] = Vec3::Z;
    vertices[11] = -Vec3::Z;

    let phi = (0.5_f32).atan();
    for i in 0..5 {
        let theta = (i as f32) / 5.0 * TAU;
        vertices[i + 1] = Vec3::new(theta.cos() * phi.cos(), theta.sin() * phi.cos(), phi.sin());
    }

    let phi = -phi;
    for i in 0..5 {
        let theta = (i as f32 + 0.5) / 5.0 * TAU;
        vertices[i + 6] = Vec3::new(theta.cos() * phi.cos(), theta.sin() * phi.cos(), phi.sin());
    }

    vertices
}

pub static VERTICES: Lazy<[Vec3; 12]> = Lazy::new(vertices);

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

pub struct Icosahedron {
    pub model: Model,
}

impl Icosahedron {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let model = Model::with_computed_normals(
            gfx,
            Some("Icosahedron"),
            &*VERTICES,
            bytemuck::cast_slice(&INDICES),
        );
        Self { model }
    }
}
