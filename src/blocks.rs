use bevy::{prelude::*, render::mesh::VertexAttributeValues};

#[derive(Debug, Default, Clone, Copy, strum_macros::EnumIter)]
pub enum BlockType {
    #[default]
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
    let uv_x_0 = (x as f32 + 0.02) * uv_off;
    let uv_x_1 = (x as f32 + 0.98) * uv_off;
    let uv_y_0 = (y as f32 + 0.02) * uv_off;
    let uv_y_1 = (y as f32 + 0.98) * uv_off;
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
        4,5,6,6,7,4,
        8,9,10,10,11,8,
        12,13,14,14,15,12,
        16,17,18,18,19,16,
        20,21,22,22,23,20
    ]
}

fn get_test_vertexes() -> Vec<[f32; 3]> {
    const SIZE_LENGTH: f32 = 1.0;
    const HALF_LENGTH: f32 = SIZE_LENGTH / 2.0;
    vec![
        // Front face
        [-HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],   // 0
        [HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],    // 1
        [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],     // 2
        [-HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],    // 3

        // Back face
        [HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],    // 4
        [HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH],   // 5
        [-HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH],  // 6
        [-HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],   // 7

        // Left face
        [-HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH],  // 8
        [-HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],   // 9
        [-HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],    // 10
        [-HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],   // 11

        // Right face
        [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],     // 12
        [HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],    // 13
        [HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH],   // 14
        [HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],    // 15

        // Top face
        [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],     // 16
        [HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],    // 17
        [-HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],   // 18
        [-HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],    // 19

        // Bottom face
        [-HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH],  // 20
        [HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH],   // 21
        [HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],    // 22
        [-HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],   // 23
    ]
}

impl rand::prelude::Distribution<BlockType> for BlockType {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> BlockType {
        // match rng.gen_range(0, 3) { // rand 0.5, 0.6, 0.7
        match rng.gen_range(0..=8) { // rand 0.8
            0 => BlockType::Bedrock,
            1 => BlockType::Gravel,
            2 => BlockType::Sand,
            3 => BlockType::Dirt,
            4 => BlockType::Stone,
            5 => BlockType::GoldOre,
            6 => BlockType::IronOre,
            7 => BlockType::CoalOre,
            8 => BlockType::DeadBush,
            _ => BlockType::Air,
        }
    }
}