use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use once_cell::sync::Lazy;
use wgpu::util::DeviceExt;

use crate::GraphicsContext;

pub static VERTEX_ATTRIBUTES: Lazy<[wgpu::VertexAttribute; 2]> = Lazy::new(|| {
    wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
    ]
});

pub const INDEX_FORMAT: wgpu::IndexFormat = wgpu::IndexFormat::Uint16;

#[derive(Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

pub struct Model {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_range: Range<u32>,
}

impl Model {
    pub fn new(
        gfx: &GraphicsContext,
        label: Option<&'static str>,
        vertices: &[Vertex],
        tris: &[u16],
    ) -> Self {
        let vertex_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(tris),
                usage: wgpu::BufferUsages::INDEX,
            });

        let index_range = 0..u32::try_from(tris.len()).unwrap();

        Self {
            vertex_buffer,
            index_buffer,
            index_range,
        }
    }
}
