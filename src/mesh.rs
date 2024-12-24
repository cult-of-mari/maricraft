use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

const TEXTURE_SIZE: f32 = 16.0;

fn block_uv(index: u32) -> [Vec2; 2] {
    let position = index as f32 * Vec2::X;

    let min = position / TEXTURE_SIZE;
    let max = (position + Vec2::ONE) / TEXTURE_SIZE;

    [min, max]
}

pub fn new_block(front: u32, back: u32, right: u32, left: u32, top: u32, bottom: u32) -> Mesh {
    let [front_min, front_max] = block_uv(front);
    let [back_min, back_max] = block_uv(back);
    let [right_min, right_max] = block_uv(right);
    let [left_min, left_max] = block_uv(left);
    let [top_min, top_max] = block_uv(top);
    let [bottom_min, bottom_max] = block_uv(bottom);

    let min = -0.5;
    let max = 0.5;

    let vertices = &[
        // Front
        ([min, min, max], [0.0, 0.0, 1.0], [front_min.x, front_max.y]),
        ([max, min, max], [0.0, 0.0, 1.0], [front_max.x, front_max.y]),
        ([max, max, max], [0.0, 0.0, 1.0], [front_max.x, front_min.y]),
        ([min, max, max], [0.0, 0.0, 1.0], [front_min.x, front_min.y]),
        // Back
        ([min, max, min], [0.0, 0.0, -1.0], [back_max.x, back_min.y]),
        ([max, max, min], [0.0, 0.0, -1.0], [back_min.x, back_min.y]),
        ([max, min, min], [0.0, 0.0, -1.0], [back_min.x, back_max.y]),
        ([min, min, min], [0.0, 0.0, -1.0], [back_max.x, back_max.y]),
        // Right
        ([max, min, min], [1.0, 0.0, 0.0], [right_max.x, right_max.y]),
        ([max, max, min], [1.0, 0.0, 0.0], [right_max.x, right_min.y]),
        ([max, max, max], [1.0, 0.0, 0.0], [right_min.x, right_min.y]),
        ([max, min, max], [1.0, 0.0, 0.0], [right_min.x, right_max.y]),
        // Left
        ([min, min, max], [-1.0, 0.0, 0.0], [left_max.x, left_max.y]),
        ([min, max, max], [-1.0, 0.0, 0.0], [left_max.x, left_min.y]),
        ([min, max, min], [-1.0, 0.0, 0.0], [left_min.x, left_min.y]),
        ([min, min, min], [-1.0, 0.0, 0.0], [left_min.x, left_max.y]),
        // Top
        ([max, max, min], [0.0, 1.0, 0.0], [top_max.x, top_min.y]),
        ([min, max, min], [0.0, 1.0, 0.0], [top_min.x, top_min.y]),
        ([min, max, max], [0.0, 1.0, 0.0], [top_min.x, top_max.y]),
        ([max, max, max], [0.0, 1.0, 0.0], [top_max.x, top_max.y]),
        // Bottom
        (
            [max, min, max],
            [0.0, -1.0, 0.0],
            [bottom_max.x, bottom_max.y],
        ),
        (
            [min, min, max],
            [0.0, -1.0, 0.0],
            [bottom_max.x, bottom_min.y],
        ),
        (
            [min, min, min],
            [0.0, -1.0, 0.0],
            [bottom_min.x, bottom_min.y],
        ),
        (
            [max, min, min],
            [0.0, -1.0, 0.0],
            [bottom_min.x, bottom_max.y],
        ),
    ];

    let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
    let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

    let indices = Indices::U32(vec![
        0, 1, 2, 2, 3, 0, // Front
        4, 5, 6, 6, 7, 4, // Back
        8, 9, 10, 10, 11, 8, // Right
        12, 13, 14, 14, 15, 12, // Left
        16, 17, 18, 18, 19, 16, // Top
        20, 21, 22, 22, 23, 20, // Bottom
    ]);

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(indices)
}
