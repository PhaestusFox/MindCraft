use std::sync::{Arc, RwLock};

use super::{BlockType, ChunkData};
use crate::prelude::*;
use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::Indices, utils::HashMap};
use noise::NoiseFn;
use rand::Rng;
use strum::IntoEnumIterator;

#[derive(Clone)]
pub struct Chunk {
    blocks: [BlockType; CHUNK_VOLUME as usize],
}

impl Chunk {
    pub const EMPTY: Chunk = Chunk {
        blocks: [BlockType::Air; CHUNK_VOLUME as usize],
    };

    pub async fn new<T: NoiseFn<f64, 2>>(
        noise: T,
        id: IVec3,
        mut rng: impl Rng,
    ) -> Result<Chunk, GenError> {
        use rand::distributions::Distribution;
        let mut chunk = [BlockType::Air; CHUNK_VOLUME as usize];
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let rng_0 = rng.gen_range(0..=4);
                let rng_1 = rng.gen_range(2..=4);
                let chunk_y = id.y * CHUNK_SIZE;
                let chunk_x = id.x * CHUNK_SIZE + x;
                let chunk_z = id.z * CHUNK_SIZE + z;
                let height =
                    noise.get([chunk_x as f64 * JIGGLE, chunk_z as f64 * JIGGLE]) / 2. + 0.5;
                let height = (height * GROUND_HEIGHT) as i32;
                for y in 0..CHUNK_SIZE {
                    let current_height = chunk_y + y;
                    // at bedrock
                    chunk[Chunk::index(x, y, z)] = if current_height == 0 {
                        BlockType::Bedrock
                        // above ground
                    } else if current_height == height {
                        BlockType::Grass
                    } else if height - current_height < 4 && height - current_height > 0 {
                        BlockType::Dirt
                    } else if current_height > height {
                        BlockType::Air
                    } else {
                        BlockType::Stone
                    }
                }
            }
        }
        Ok(Chunk { blocks: chunk })
    }

    pub async fn gen_mesh(
        id: ChunkId,
        data: Arc<RwLock<ChunkData>>,
        atlas: TextureHandles,
    ) -> Result<Mesh, MeshError> {
        let mut positions = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();
        let mut color: Vec<[f32; 4]> = Vec::new();
        let chunk = data
            .read()
            .unwrap()
            .try_get(&id)
            .ok_or(MeshError::ChunkNotGenerated(id))?
            .clone();
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let block = chunk.get_block(x, y, z);
                    if let BlockType::Air = block {
                        continue;
                    }
                    for direction in Direction::iter() {
                        let mut neighbor = IVec3::new(x, y, z) + direction;
                        let neighbor = if in_chunk(neighbor) {
                            chunk.blocks[Chunk::index(neighbor.x, neighbor.y, neighbor.z)]
                        } else {
                            if neighbor.y < 0 {
                                neighbor.y += CHUNK_SIZE
                            } else if neighbor.x < 0 {
                                neighbor.x += CHUNK_SIZE
                            } else if neighbor.z < 0 {
                                neighbor.z += CHUNK_SIZE
                            }
                            let id = id.0 + direction;
                            data.read().unwrap().get(&ChunkId(id)).blocks[Chunk::index(
                                neighbor.x % CHUNK_SIZE,
                                neighbor.y % CHUNK_SIZE,
                                neighbor.z % CHUNK_SIZE,
                            )]
                        };

                        if neighbor.is_transparent() {
                            let block = block.gen_mesh(direction, &atlas);
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
        let mut mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        );
        mesh.insert_indices(Indices::U32(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 1., 0.]; positions.len()]);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, color);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        Ok(mesh)
    }

    #[inline(always)]
    fn index(x: i32, y: i32, z: i32) -> usize {
        (x + z * CHUNK_SIZE + y * CHUNK_AREA) as usize
    }

    pub fn get_block(&self, mut x: i32, mut y: i32, mut z: i32) -> BlockType {
        x %= CHUNK_SIZE;
        y %= CHUNK_SIZE;
        z %= CHUNK_SIZE;
        self.blocks
            .get(Chunk::index(x, y, z))
            .copied()
            .unwrap_or(BlockType::Air)
    }
}

#[inline(always)]
fn in_chunk(pos: IVec3) -> bool {
    pos.y < CHUNK_SIZE
        && pos.y >= 0
        && pos.x < CHUNK_SIZE
        && pos.x >= 0
        && pos.z < CHUNK_SIZE
        && pos.z >= 0
}

#[derive(Debug, thiserror::Error)]
pub enum GenError {}

#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("No Chunk Data for {:0?} was found", 0)]
    ChunkNotGenerated(ChunkId),
}
