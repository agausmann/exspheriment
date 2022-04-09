mod icosahedron;

pub use self::icosahedron::Icosahedron;

use crate::{
    model::{Model, Vertex},
    GraphicsContext,
};

pub struct Triangle {
    pub model: Model,
}

impl Triangle {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let model = Model::new(
            gfx,
            Some("Triangle"),
            &[
                Vertex {
                    position: [-1.0, 0.0, -1.0],
                    normal: [0.0, -1.0, 0.0],
                },
                Vertex {
                    position: [0.0, 0.0, 1.0],
                    normal: [0.0, -1.0, 0.0],
                },
                Vertex {
                    position: [1.0, 0.0, -1.0],
                    normal: [0.0, -1.0, 0.0],
                },
            ],
            &[0, 1, 2],
        );

        Self { model }
    }
}

pub struct Square {
    pub model: Model,
}

impl Square {
    pub fn new(gfx: &GraphicsContext) -> Self {
        let model = Model::new(
            gfx,
            Some("Square"),
            &[
                Vertex {
                    position: [-1.0, -1.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [-1.0, 1.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
            ],
            &[0, 1, 2, 2, 3, 0],
        );

        Self { model }
    }
}
