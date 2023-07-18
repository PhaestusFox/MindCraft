use bevy::{prelude::*, render::mesh::VertexAttributeValues};

pub enum BlockType {
    Air,
    Bedrock,
    Gravel,
    Dirt,
    Stone,
    Sand,
    GoldOre,
    IronOre,
    CoalOre,
    DeadBush,
}

impl BlockType {
    pub const fn get_texture_paths(&self) -> &'static [&'static str] {
        match self {
            BlockType::Air => &[],
            BlockType::Bedrock => &["PureBDcraft/textures/block/bedrock.png"],
            BlockType::Gravel => &["PureBDcraft/textures/block/gravel.png"],
            BlockType::Dirt => &["PureBDcraft/textures/block/dirt.png"],
            BlockType::Stone => &["PureBDcraft/textures/block/stone.png"],
            BlockType::Sand => &["PureBDcraft/textures/block/sand.png"],
            BlockType::GoldOre => &["PureBDcraft/textures/block/gold_ore.png"],
            BlockType::IronOre => &["PureBDcraft/textures/block/iron_ore.png"],
            BlockType::CoalOre => &["PureBDcraft/textures/block/coal_ore.png"],
            BlockType::DeadBush => &["PureBDcraft/textures/block/dead_bush.png"],
        }
    }
}

pub fn make_test_block_mesh(
    block_index: usize,
    atlas_size: usize
) -> Mesh {
    assert!(block_index < atlas_size * atlas_size);
    let y = block_index / atlas_size;
    let x = block_index - y * atlas_size;
    let uv_off = 1.0 / atlas_size as f32;
    let uv_x_0 =  x as f32 * uv_off;
    let uv_x_1 = (x as f32 + 0.99)  * uv_off;
    let uv_y_0 =  y as f32 * uv_off;
    let uv_y_1 = (y as f32 + 0.99)  * uv_off;
    let mut uvs = Vec::with_capacity(24);
    for _ in 0..6 {
        uvs.extend([
            [uv_x_0, uv_y_1],
            [uv_x_1, uv_y_1],
            [uv_x_1, uv_y_0],
            [uv_x_0, uv_y_0],
        ]);
    };
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, VertexAttributeValues::Float32x3(get_test_vertexes()));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    mesh.set_indices(Some(bevy::render::mesh::Indices::U16(get_test_indices())));
    mesh
}

fn get_test_indices() -> Vec<u16> {
    vec![
        0,1,2,2,3,0,
        6,5,4,4,7,6,
        8,9,10,10,11,8,
        14,13,12,12,15,14,
        18,17,16,16,19,18,
        20,21,22,22,23,20
    ]
}

fn get_test_vertexes() -> Vec<[f32; 3]> {
    const side_length: f32 = 1.0;
    const half_length: f32 = side_length / 2.0;
    vec![
        // Front face
        [-half_length, -half_length, half_length],   // 0
        [half_length, -half_length, half_length],    // 1
        [half_length, half_length, half_length],     // 2
        [-half_length, half_length, half_length],    // 3

        // Back face
        [-half_length, -half_length, -half_length],  // 4
        [half_length, -half_length, -half_length],   // 5
        [half_length, half_length, -half_length],    // 6
        [-half_length, half_length, -half_length],   // 7

        // Left face
        [-half_length, -half_length, -half_length],  // 8
        [-half_length, -half_length, half_length],   // 9
        [-half_length, half_length, half_length],    // 10
        [-half_length, half_length, -half_length],   // 11

        // Right face
        [half_length, -half_length, -half_length],   // 12
        [half_length, -half_length, half_length],    // 13
        [half_length, half_length, half_length],     // 14
        [half_length, half_length, -half_length],    // 15

        // Top face
        [-half_length, half_length, -half_length],   // 16
        [half_length, half_length, -half_length],    // 17
        [half_length, half_length, half_length],     // 18
        [-half_length, half_length, half_length],    // 19

        // Bottom face
        [-half_length, -half_length, -half_length],  // 20
        [half_length, -half_length, -half_length],   // 21
        [half_length, -half_length, half_length],    // 22
        [-half_length, -half_length, half_length],   // 23
    ]
}