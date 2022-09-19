use bevy::{
    math::Vec3,
    prelude::Mesh,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::geometry::icosahedron;

pub struct Icosphere {
    pub radius: f32,
    pub subdivisions: usize,
    pub method: SubdivisionMethod,
}

impl Default for Icosphere {
    fn default() -> Self {
        Self {
            radius: 1.0,
            subdivisions: 1,
            method: SubdivisionMethod::Lerp,
        }
    }
}

pub enum SubdivisionMethod {
    Lerp,
    Slerp,
}

impl SubdivisionMethod {
    fn interpolate(&self, a: impl Into<Vec3>, b: impl Into<Vec3>, t: f32) -> Vec3 {
        match self {
            Self::Lerp => Vec3::lerp(a.into(), b.into(), t).into(),
            Self::Slerp => crate::math::slerp(a.into(), b.into(), t).into(),
        }
    }
}

impl From<Icosphere> for Mesh {
    fn from(ico: Icosphere) -> Self {
        use icosahedron::{INDICES, VERTICES};

        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut triangles = Vec::new();

        for triangle in INDICES.chunks_exact(3) {
            let up = VERTICES[triangle[0] as usize];
            let right = VERTICES[triangle[1] as usize];
            let left = VERTICES[triangle[2] as usize];

            let mut last_row = Vec::new();
            last_row.push((ico.radius * Vec3::from(up)).into());

            let mut this_row = Vec::new();

            for row in 1..=ico.subdivisions {
                // Construct vertexes for this row
                let factor = row as f32 / ico.subdivisions as f32;
                let row_start = ico.method.interpolate(up, left, factor);
                let row_end = ico.method.interpolate(up, right, factor);
                for col in 0..=row {
                    let col_factor = col as f32 / row as f32;
                    let unit_vertex = ico
                        .method
                        .interpolate(row_start, row_end, col_factor)
                        .normalize();
                    this_row.push((ico.radius * unit_vertex).into());
                }

                // Construct triangles for this row
                let last_row_base = vertices.len();
                let this_row_base = last_row_base + last_row.len();
                for i in 0..row {
                    triangles.extend([
                        u16::try_from(this_row_base + i).unwrap(),
                        u16::try_from(this_row_base + i + 1).unwrap(),
                        u16::try_from(last_row_base + i).unwrap(),
                    ])
                }

                for i in 0..row - 1 {
                    triangles.extend([
                        u16::try_from(last_row_base + i).unwrap(),
                        u16::try_from(this_row_base + i + 1).unwrap(),
                        u16::try_from(last_row_base + i + 1).unwrap(),
                    ])
                }

                // Shift rows
                vertices.append(&mut last_row);
                last_row.append(&mut this_row);
            }

            // Shift in last row
            vertices.append(&mut last_row)
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 0.0]; vertices.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.set_indices(Some(Indices::U16(triangles.into())));
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
        mesh
    }
}
