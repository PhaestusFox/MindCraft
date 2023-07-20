use bevy::prelude::*;

use crate::prelude::*;

#[derive(Component, Clone, Copy)]
struct ChunkId(IVec3);

#[derive(Component, Clone, Copy)]
struct BlockId(IVec3);

impl From<BlockId> for ChunkId {
    fn from(value: BlockId) -> Self {
        ChunkId(
            IVec3::new(value.0.x / CHUNK_SIZE, value.0.y / CHUNK_SIZE, value.0.z / CHUNK_SIZE)
        )
    }
}