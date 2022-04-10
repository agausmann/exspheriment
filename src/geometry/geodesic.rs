use std::cmp::Ordering;

use glam::Vec3;

use crate::{geometry::icosahedron, math::slerp, model::Model, GraphicsContext};

pub struct Geodesic {
    pub subdivisions: usize,
    pub model: Model,
}

impl Geodesic {
    fn using_lerp(
        gfx: &GraphicsContext,
        subdivisions: usize,
        lerp: fn(Vec3, Vec3, f32) -> Vec3,
    ) -> Self {
        use icosahedron::{INDICES, VERTICES};

        let mut vertices = Vec::new();
        let mut triangles = Vec::new();

        for triangle in INDICES.chunks_exact(3) {
            let up = VERTICES[triangle[0] as usize];
            let right = VERTICES[triangle[1] as usize];
            let left = VERTICES[triangle[2] as usize];

            let mut last_row = Vec::new();
            last_row.push(up);

            let mut this_row = Vec::new();

            for row in 1..=subdivisions {
                // Construct vertexes for this row
                let factor = row as f32 / subdivisions as f32;
                let row_start = lerp(up, left, factor);
                let row_end = lerp(up, right, factor);
                for col in 0..=row {
                    let col_factor = col as f32 / row as f32;
                    this_row.push(lerp(row_start, row_end, col_factor).normalize());
                }

                // Construct triangles for this row
                let last_row_base = vertices.len();
                let this_row_base = last_row_base + last_row.len();
                for i in 0..row {
                    triangles.extend([
                        u16::try_from(this_row_base + i).unwrap(),
                        u16::try_from(last_row_base + i).unwrap(),
                        u16::try_from(this_row_base + i + 1).unwrap(),
                    ])
                }

                for i in 0..row - 1 {
                    triangles.extend([
                        u16::try_from(last_row_base + i).unwrap(),
                        u16::try_from(last_row_base + i + 1).unwrap(),
                        u16::try_from(this_row_base + i + 1).unwrap(),
                    ])
                }

                // Shift rows
                vertices.append(&mut last_row);
                last_row.append(&mut this_row);
            }

            // Shift in last row
            vertices.append(&mut last_row)
        }

        let model = Model::with_computed_normals(gfx, Some("Geodesic"), &vertices, &triangles);
        Self {
            subdivisions,
            model,
        }
    }

    pub fn new(gfx: &GraphicsContext, subdivisions: usize) -> Self {
        Self::using_lerp(gfx, subdivisions, Vec3::lerp)
    }

    pub fn with_slerp(gfx: &GraphicsContext, subdivisions: usize) -> Self {
        Self::using_lerp(gfx, subdivisions, crate::math::slerp)
    }
}
