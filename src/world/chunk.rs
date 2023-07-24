use crate::prelude::*;
use bevy::{prelude::*, render::mesh::Indices, utils::HashMap};
use bevy_rapier3d::prelude::Collider;
use noise::NoiseFn;
use rand::{Rng, SeedableRng};
use strum::IntoEnumIterator;

#[derive(Clone, Copy)]
pub struct Chunk {
    pub id: ChunkId,
    pub entity: Option<Entity>,
    pub blocks: [BlockType; CHUNK_VOL as usize]
}

pub struct ChunkGenData {
    pub main_mesh: Option<Mesh>,
    pub water_mesh: Option<Mesh>,
    pub collider: Option<Collider>,
}

impl Chunk {
    pub async fn make_mesh(id: ChunkId, map: super::Map, atlas_map: TextureHandles) -> Option<ChunkGenData> {
        let map = map.0.read().unwrap();
        let chunk = map.get_chunk(&id)?;
        Some(ChunkGenData {
            main_mesh: chunk.gen_mesh(&atlas_map),
            water_mesh: chunk.gen_water_mesh(&atlas_map),
            collider: chunk.gen_collider()
        })
    }

    pub fn set_entity(&mut self, entity: Entity) {
        self.entity = Some(entity);
    }

    pub fn new(pos: ChunkId, noise: &noise::Fbm<noise::SuperSimplex>, seed: u64) -> Chunk {
        use rand::distributions::Distribution;
        let mut chunk = [BlockType::Air; CHUNK_VOL as usize];
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let rng_0 = rng.gen_range(0..=4);
                let rng_1 = rng.gen_range(2..=4);
                let chunk_y = pos.y() * CHUNK_SIZE;
                let chunk_x = pos.x() * CHUNK_SIZE + x;
                let chunk_z = pos.z() * CHUNK_SIZE + z;
                let height =
                    noise.get([chunk_x as f64 * JIGGLE, chunk_z as f64 * JIGGLE]) / 2. + 0.5;
                let height = (height * GROUND_HEIGHT) as i32;
                for y in 0..CHUNK_SIZE {
                    let current_height = chunk_y + y;
                    if current_height < height/ 2 {
                        chunk[(y * CHUNK_AREA + z * CHUNK_SIZE + x) as usize] = BlockType::Water;
                    }
                    // at bedrock
                    chunk[(y * CHUNK_AREA + z * CHUNK_SIZE + x) as usize] = if current_height == 0 {
                        BlockType::Bedrock
                        // above ground
                    } else if current_height > height {
                        // repace air with water if blow water level
                        if current_height < WATER_LEVEL {
                            BlockType::Water
                        } else {
                            break;
                        }
                    // upto 3 block above water level and 3 block below ground
                    } else if current_height - WATER_LEVEL < rng_0 && (height - current_height).abs() < rng_1 + 1 {
                        BlockType::Sand
                    }
                    // top layer of ground
                    else if current_height == height {
                        BlockType::Grass
                    // top 3 layers of ground
                    } else if height - current_height < rng_1 {
                        BlockType::Dirt
                    // pro of depth
                    } else if rng.gen_bool(1. / (current_height + 1) as f64) {
                        BlockType::Air.sample(&mut rng)
                    // nothing specil default
                    } else {
                        BlockType::Stone
                    }
                }
            }
        }
        Chunk {
            id: pos,
            entity: None,
            blocks: chunk
        }
    }

    fn get_block(&self, pos: ChunkId) -> BlockType {
        if pos > CHUNK_SIZE - 1 || pos < 0 {
            return BlockType::Air;
        }
        self.blocks[(pos.y() * CHUNK_AREA + pos.z() * CHUNK_SIZE + pos.x()) as usize]
    }

    pub fn gen_mesh(&self, atlas_map: &TextureHandles) -> Option<Mesh> {
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        let mut color: Vec<[f32; 4]> = Vec::new();
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let current = ChunkId::new(x, y, z);
                    let block = self.get_block(current);
                    match block {
                        BlockType::Air | BlockType::Water => continue,
                        _ => {}
                    }
                    for direction in Direction::iter() {
                        if self.get_block(current.get(direction)).is_transparent() {
                            let block = block.gen_mesh(direction, atlas_map);
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
        if indices.is_empty() {
            return None;
        }
        let mut mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; positions.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, color);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        Some(mesh)
    }

    pub fn gen_water_mesh(&self, atlas_map: &TextureHandles) -> Option<Mesh> {
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        let mut color: Vec<[f32; 4]> = Vec::new();
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let current = ChunkId::new(x, y, z);
                    let block = self.get_block(current);
                    if let BlockType::Water = block {
                        for direction in Direction::iter() {
                            if let BlockType::Air = self.get_block(current.get(direction)) {
                                let is_top_air = self.get_block(current.get(Direction::Up)) == BlockType::Air;
                                let block = BlockType::water_mesh(direction, atlas_map, is_top_air);
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
        }
        if indices.is_empty() {
            return None;
        }
        let mut mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; positions.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, color);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        Some(mesh)
    }

    pub fn gen_collider(&self) -> Option<bevy_rapier3d::prelude::Collider> {
        use bevy_rapier3d::prelude::*;
        let mut vertexs: Vec<bevy_rapier3d::na::OPoint<f32, bevy_rapier3d::na::Const<3>>> = Vec::new();
        let mut indices = Vec::new();
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let current = ChunkId::new(x, y, z);
                    let block = self.get_block(current);
                    match block {
                        BlockType::Air | BlockType::Water => continue,
                        _ => {}
                    }
                    for direction in Direction::iter() {
                        if self.get_block(current.get(direction)).is_solid() {
                            continue;
                        }
                        let ind = [[0, 1, 2], [2, 3, 0]];
                        let plane = direction.perp();
                        if self.get_block(current.get(plane.rev())).is_solid() && !self.get_block(current.get(plane.rev()).get(direction)).is_solid() {
                            continue;
                        }
                        indices.extend(
                            ind.map(|mut val| {
                                val[0] += vertexs.len() as u32;
                                val[1] += vertexs.len() as u32;
                                val[2] += vertexs.len() as u32;
                                val
                            })
                        );
                        let mut len = 0;
                        let mut next = current.get(plane);
                        while self.get_block(next).is_solid() {
                            len += 1;
                            next = next.get(plane);
                        }
                        vertexs.extend(BlockType::block_face_with_len(direction, len as f32)     // 3
                            .into_iter().map(|pos| {
                                let new_pos: bevy_rapier3d::na::OPoint<f32, bevy_rapier3d::na::Const<3>> = [pos[0] + x as f32, pos[1] + y as f32, pos[2] + z as f32].into();
                                new_pos
                            }));
                        }
                    }
                }
            }
            if vertexs.is_empty() {
                return None;
            }
            Some(bevy_rapier3d::rapier::prelude::SharedShape::trimesh_with_flags(vertexs, indices, TriMeshFlags::MERGE_DUPLICATE_VERTICES).into())
        }
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

impl Direction {
    fn perp(&self) -> Self {
        match self {
            Direction::Up => Direction::Forward,
            Direction::Down => Direction::Back,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
            Direction::Forward => Direction::Right,
            Direction::Back => Direction::Left,
        }
    }

    fn rev(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Forward => Direction::Back,
            Direction::Back => Direction::Forward,
        }
    }
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
