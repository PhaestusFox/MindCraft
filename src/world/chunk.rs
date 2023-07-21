use crate::prelude::*;
use bevy::{prelude::*, render::mesh::Indices, utils::HashMap};
use noise::NoiseFn;
use rand::{Rng, SeedableRng};
use strum::IntoEnumIterator;

#[derive(Component)]
pub struct Chunk([BlockType; CHUNK_VOL as usize]);

impl Chunk {
    pub fn new(pos: ChunkId, noise: &noise::Fbm<noise::SuperSimplex>, seed: u64) -> Chunk {
        use rand::distributions::Distribution;
        let mut chunk = [BlockType::Air; CHUNK_VOL as usize];
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let current_height = pos.y() * CHUNK_SIZE;
                let current_x = pos.x() * CHUNK_SIZE + x;
                let current_z = pos.z() * CHUNK_SIZE + z;
                let height =
                    noise.get([current_x as f64 * JIGGLE, current_z as f64 * JIGGLE]) / 2. + 0.5;
                max = max.max(height);
                min = min.min(height);
                let height = (height * GROUND_HEIGHT) as i32;
                for y in 0..CHUNK_SIZE {
                    if y + current_height > height {
                        break;
                    }
                    chunk[(y * CHUNK_AREA + z * CHUNK_SIZE + x) as usize] =
                        BlockType::Air.sample(&mut rng);
                }
            }
        }
        println!("Chunk {pos:?}, min: {min}, max: {max}");
        Chunk(chunk)
    }

    fn get_block(&self, pos: ChunkId) -> BlockType {
        if pos > CHUNK_SIZE - 1 || pos < 0 {
            return BlockType::Air;
        }
        self.0[(pos.y() * CHUNK_AREA + pos.z() * CHUNK_SIZE + pos.x()) as usize]
    }

    pub fn gen_mesh(&self, atlas_map: &TextureHandles) -> Mesh {
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        let mut color: Vec<[f32; 4]> = Vec::new();
        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let current = ChunkId::new(x, y, z);
                    let block = self.get_block(current);
                    if let BlockType::Air = block {
                        continue;
                    }
                    for direction in Direction::iter() {
                        let block = block.gen_mesh(direction, atlas_map);
                        if let BlockType::Air = self.get_block(current.get(direction)) {
                            indices.extend(
                                block
                                    .indices
                                    .into_iter()
                                    .map(|i| *i + positions.len() as u32),
                            );
                            positions.extend(block.pos.into_iter().map(|pos| {
                                [pos[0] + x as f32, pos[1] + y as f32, pos[2] + z as f32]
                            }));
                            uvs.extend(block.uv.into_iter());
                            color.extend(block.color.into_iter());
                        }
                    }
                }
            }
        }
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, color);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Position {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl PartialEq<isize> for Position {
    fn eq(&self, other: &isize) -> bool {
        self.x == *other && self.y == *other && self.z == *other
    }
}

impl PartialOrd<isize> for Position {
    fn lt(&self, other: &isize) -> bool {
        self.x < *other || self.y < *other || self.z < *other
    }
    fn gt(&self, other: &isize) -> bool {
        self.x > *other || self.y > *other || self.z > *other
    }
    fn partial_cmp(&self, _: &isize) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Position {
    pub fn new(x: isize, y: isize, z: isize) -> Position {
        Position { x, y, z }
    }
    fn get(&self, direction: Direction) -> Position {
        match direction {
            Direction::Up => Position {
                x: self.x,
                y: self.y + 1,
                z: self.z,
            },
            Direction::Down => Position {
                x: self.x,
                y: self.y - 1,
                z: self.z,
            },
            Direction::Left => Position {
                x: self.x - 1,
                y: self.y,
                z: self.z,
            },
            Direction::Right => Position {
                x: self.x + 1,
                y: self.y,
                z: self.z,
            },
            Direction::Forward => Position {
                x: self.x,
                y: self.y,
                z: self.z + 1,
            },
            Direction::Back => Position {
                x: self.x,
                y: self.y,
                z: self.z - 1,
            },
        }
    }
}

fn get_uv(block: usize, atlas_size: usize) -> [[f32; 2]; 4] {
    let y = block / atlas_size;
    let x = block - y * atlas_size;
    let uv_off = 1.0 / atlas_size as f32;
    let uv_x_0 = (x as f32 + 0.02) * uv_off;
    let uv_x_1 = (x as f32 + 0.98) * uv_off;
    let uv_y_0 = (y as f32 + 0.02) * uv_off;
    let uv_y_1 = (y as f32 + 0.98) * uv_off;
    [
        [uv_x_0, uv_y_1],
        [uv_x_1, uv_y_1],
        [uv_x_1, uv_y_0],
        [uv_x_0, uv_y_0],
    ]
}

#[derive(Debug, strum_macros::EnumIter, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Forward,
    Back,
}

const fn gen_face(direction: Direction) -> [[f32; 3]; 4] {
    const SIZE_LENGTH: f32 = 1.0;
    const HALF_LENGTH: f32 = SIZE_LENGTH / 2.0;
    const NEG_HALF_LENGTH: f32 = -HALF_LENGTH;
    match direction {
        Direction::Forward => [
            // Front face
            [NEG_HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH], // 0
            [HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH],     // 1
            [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],         // 2
            [NEG_HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],     // 3
        ],
        Direction::Back => [
            // Back face
            [HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 4
            [NEG_HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 5
            [NEG_HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH], // 6
            [HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH],     // 7
        ],
        Direction::Left => [
            // Left face
            [NEG_HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 8
            [NEG_HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH],     // 9
            [NEG_HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],         // 10
            [NEG_HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH],     // 11
        ],
        Direction::Right => [
            // Right face
            [HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH], // 12
            [HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 13
            [HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH], // 14
            [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],     // 15
        ],
        Direction::Up => [
            // Top face
            [HALF_LENGTH, HALF_LENGTH, HALF_LENGTH],     // 16
            [HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH], // 17
            [NEG_HALF_LENGTH, HALF_LENGTH, NEG_HALF_LENGTH], // 18
            [NEG_HALF_LENGTH, HALF_LENGTH, HALF_LENGTH], // 19
        ],
        Direction::Down => [
            // Bottom face
            [NEG_HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH], // 20
            [HALF_LENGTH, NEG_HALF_LENGTH, NEG_HALF_LENGTH],     // 21
            [HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH],         // 22
            [NEG_HALF_LENGTH, NEG_HALF_LENGTH, HALF_LENGTH],     // 23
        ],
    }
}

const fn get_indices(offset: u32) -> [u32; 6] {
    [
        offset + 0,
        offset + 1,
        offset + 2,
        offset + 2,
        offset + 3,
        offset + 0,
    ]
}
