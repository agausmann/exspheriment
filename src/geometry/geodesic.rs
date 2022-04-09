use crate::{geometry::icosahedron, model::Model, GraphicsContext};

pub struct Geodesic {
    pub subdivisions: usize,
    pub model: Model,
}

impl Geodesic {
    pub fn new(gfx: &GraphicsContext, subdivisions: usize) -> Self {
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
                let row_start = up.lerp(left, factor);
                let row_end = up.lerp(right, factor);
                for col in 0..=row {
                    let col_factor = col as f32 / row as f32;
                    this_row.push(row_start.lerp(row_end, col_factor).normalize());
                }

                // Construct triangles for this row
                let last_row_base = vertices.len();
                let this_row_base = last_row_base + last_row.len();
                for i in 0..row {
                    triangles.push([
                        u16::try_from(this_row_base + i).unwrap(),
                        u16::try_from(last_row_base + i).unwrap(),
                        u16::try_from(this_row_base + i + 1).unwrap(),
                    ])
                }

                for i in 0..row - 1 {
                    triangles.push([
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

        let model = Model::with_computed_normals(
            gfx,
            Some("Geodesic"),
            &vertices,
            bytemuck::cast_slice(&triangles),
        );
        Self {
            subdivisions,
            model,
        }
    }
}
