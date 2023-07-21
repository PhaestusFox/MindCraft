use crate::prelude::Direction;
use bevy::{prelude::*, render::mesh::VertexAttributeValues, utils::HashMap};

#[derive(Debug, Default, Clone, Copy, strum_macros::EnumIter, PartialEq, Eq, Hash)]
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
    Grass,
}

pub struct MeshData {
    pub pos: &'static [[f32; 3]],
    pub uv: Vec<[f32; 2]>,
    pub color: &'static [[f32; 4]],
    pub indices: &'static [u32],
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
            BlockType::Grass => &[
                "PureBDcraft/textures/block/grass_block_top.png",
                "PureBDcraft/textures/block/grass_block_side.png",
                "PureBDcraft/textures/block/dirt.png",
            ],
        }
    }
    pub fn gen_mesh(
        &self,
        direction: crate::world::chunk::Direction,
        atlas_map: &crate::prelude::TextureHandles,
    ) -> MeshData {
        match self {
            BlockType::Air => MeshData {
                pos: &[],
                uv: vec![],
                color: &[],
                indices: &[],
            },
            BlockType::Bedrock
            | BlockType::Gravel
            | BlockType::Dirt
            | BlockType::Stone
            | BlockType::Sand
            | BlockType::GoldOre
            | BlockType::IronOre
            | BlockType::CoalOre => {
                let index = atlas_map.get_indexes(self);
                BlockType::block_mesh(direction, index[0], atlas_map.len())
            }
            BlockType::DeadBush => todo!(),
            BlockType::Grass => BlockType::grass_mesh(direction, atlas_map),
        }
    }

    fn grass_mesh(direction: Direction, atlas_map: &crate::prelude::TextureHandles) -> MeshData {
        let indexes = atlas_map.get_indexes(&BlockType::Grass);
        match direction {
            Direction::Up => MeshData {
                pos: BlockType::block_face(direction),
                uv: BlockType::block_uv(indexes[0], atlas_map.len()),
                color: &[[0., 1., 0., 1.]; 4],
                indices: &[0, 1, 2, 2, 3, 0],
            },
            Direction::Down => MeshData {
                pos: BlockType::block_face(direction),
                uv: BlockType::block_uv(indexes[2], atlas_map.len()),
                color: &[[1., 1., 1., 1.]; 4],
                indices: &[0, 1, 2, 2, 3, 0],
            },
            _ => MeshData {
                pos: BlockType::block_face(direction),
                uv: BlockType::block_uv(indexes[1], atlas_map.len()),
                color: &[[1., 1., 1., 1.]; 4],
                indices: &[0, 1, 2, 2, 3, 0],
            },
        }
    }

    fn block_uv(block: usize, atlas_size: usize) -> Vec<[f32; 2]> {
        let y = block / atlas_size;
        let x = block - y * atlas_size;
        let uv_off = 1.0 / atlas_size as f32;
        let uv_x_0 = (x as f32 + 0.02) * uv_off;
        let uv_x_1 = (x as f32 + 0.98) * uv_off;
        let uv_y_0 = (y as f32 + 0.02) * uv_off;
        let uv_y_1 = (y as f32 + 0.98) * uv_off;
        vec![
            [uv_x_0, uv_y_1],
            [uv_x_1, uv_y_1],
            [uv_x_1, uv_y_0],
            [uv_x_0, uv_y_0],
        ]
    }

    fn block_mesh(direction: Direction, index_pos: usize, index_len: usize) -> MeshData {
        MeshData {
            pos: BlockType::block_face(direction),
            uv: BlockType::block_uv(index_pos, index_len),
            color: &[[1., 1., 1., 1.]; 4],
            indices: &[0, 1, 2, 2, 3, 0],
        }
    }

    const fn block_face(direction: Direction) -> &'static [[f32; 3]] {
        const SIZE_LENGTH: f32 = 1.0;
        const HALF_LENGTH: f32 = SIZE_LENGTH / 2.0;
        const NEG_HALF_LENGTH: f32 = -HALF_LENGTH;
        match direction {
            Direction::Forward => &[
                // Front face
                [NEG_HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH], // 0
                [HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH],     // 1
                [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],         // 2
                [NEG_HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],     // 3
            ],
            Direction::Back => &[
                // Back face
                [HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 5
                [NEG_HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 6
                [NEG_HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH], // 7
                [HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH],     // 4
            ],
            Direction::Left => &[
                // Left face
                [NEG_HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 8
                [NEG_HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH],     // 9
                [NEG_HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],         // 10
                [NEG_HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH],     // 11
            ],
            Direction::Right => &[
                // Right face
                [HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH], // 13
                [HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 14
                [HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH], // 15
                [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],     // 12
            ],
            Direction::Up => &[
                // Top face
                [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH], // 16
                [HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH], // 17
                [NEG_HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH], // 18
                [NEG_HALF_LENGTH, HALF_LENGTH, HALF_LENGTH], // 19
            ],
            Direction::Down => &[
                // Bottom face
                [NEG_HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 20
                [HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH],     // 21
                [HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH],         // 22
                [NEG_HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH],     // 23
            ],
        }
    }
}

pub fn make_test_block_mesh(block_index: usize, atlas_size: usize) -> Mesh {
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
    }
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(get_test_vertexes()),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float32x2(uvs));
    mesh.set_indices(Some(bevy::render::mesh::Indices::U16(get_test_indices())));
    mesh
}

fn get_test_indices() -> Vec<u16> {
    vec![
        0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16, 17,
        18, 18, 19, 16, 20, 21, 22, 22, 23, 20,
    ]
}

fn get_test_vertexes() -> Vec<[f32; 3]> {
    const SIZE_LENGTH: f32 = 1.0;
    const HALF_LENGTH: f32 = SIZE_LENGTH / 2.0;
    vec![
        // Front face
        [-HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH], // 0
        [HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],  // 1
        [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],   // 2
        [-HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],  // 3
        // Back face
        [HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],   // 4
        [HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH],  // 5
        [-HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH], // 6
        [-HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],  // 7
        // Left face
        [-HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH], // 8
        [-HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],  // 9
        [-HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],   // 10
        [-HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],  // 11
        // Right face
        [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],   // 12
        [HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],  // 13
        [HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH], // 14
        [HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],  // 15
        // Top face
        [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],   // 16
        [HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH],  // 17
        [-HALF_LENGTH, HALF_LENGTH, -HALF_LENGTH], // 18
        [-HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],  // 19
        // Bottom face
        [-HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH], // 20
        [HALF_LENGTH, -HALF_LENGTH, -HALF_LENGTH],  // 21
        [HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],   // 22
        [-HALF_LENGTH, -HALF_LENGTH, HALF_LENGTH],  // 23
    ]
}

impl rand::prelude::Distribution<BlockType> for BlockType {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> BlockType {
        // match rng.gen_range(0, 3) { // rand 0.5, 0.6, 0.7
        match rng.gen_range(0..=8) {
            // rand 0.8
            0 => BlockType::Bedrock,
            1 => BlockType::Gravel,
            2 => BlockType::Sand,
            3 => BlockType::Dirt,
            4 => BlockType::Stone,
            5 => BlockType::GoldOre,
            6 => BlockType::IronOre,
            7 => BlockType::CoalOre,
            8 => BlockType::Grass,
            _ => BlockType::Air,
        }
    }
}
