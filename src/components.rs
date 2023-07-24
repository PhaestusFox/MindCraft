use std::fmt::Debug;

use bevy::prelude::*;

use crate::prelude::Direction;
use crate::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkId(IVec3);

impl From<Vec3> for ChunkId {
    fn from(value: Vec3) -> Self {
        let value = value.as_ivec3();
        ChunkId(IVec3::new(
            value.x / CHUNK_SIZE,
            value.y / CHUNK_SIZE,
            value.z / CHUNK_SIZE,
        ))
    }
}

impl ChunkId {
    pub const fn new(x: i32, y: i32, z: i32) -> ChunkId {
        ChunkId(IVec3 { x, y, z })
    }

    pub const fn x(&self) -> i32 {
        self.0.x
    }

    pub const fn y(&self) -> i32 {
        self.0.y
    }

    pub const fn z(&self) -> i32 {
        self.0.z
    }

    pub const fn get(&self, direction: Direction) -> ChunkId {
        match direction {
            Direction::Up => ChunkId::new(self.x(), self.y() + 1, self.z()),
            Direction::Down => ChunkId::new(self.x(), self.y() - 1, self.z()),
            Direction::Left => ChunkId::new(self.x() - 1, self.y(), self.z()),
            Direction::Right => ChunkId::new(self.x() + 1, self.y(), self.z()),
            Direction::Forward => ChunkId::new(self.x(), self.y(), self.z() + 1),
            Direction::Back => ChunkId::new(self.x(), self.y(), self.z() - 1),
        }
    }

    /// returns the number of chunks between self and other
    /// this is the sqr distance since there is no need waste time sqruting
    pub fn sqr_distance(&self, other: ChunkId) -> i32 {
        let x_dif = self.x() - other.x();
        let y_dif = self.y() - other.y();
        let z_dif = self.z() - other.z();
        x_dif * x_dif + y_dif * y_dif + z_dif * z_dif
    }
    /// returns the number of chunks between self and other ignoring changes in y
    /// this is the sqr distance since there is no need waste time sqruting
    /// I use this for render distance since if you are too far up all the chunks unload
    pub fn flat_distance(&self, other: ChunkId) -> i32 {
        let x_dif = self.x() - other.x();
        let z_dif = self.z() - other.z();
        x_dif.abs() + z_dif.abs()
    }

    pub const fn neighbours(&self) -> [ChunkId; 6] {
        [
           self.get(Direction::Up), 
           self.get(Direction::Down), 
           self.get(Direction::Left), 
           self.get(Direction::Right), 
           self.get(Direction::Forward), 
           self.get(Direction::Back), 
        ]
    }
}

impl std::ops::Add<BlockId> for ChunkId {
    type Output = BlockId;
    fn add(self, mut rhs: BlockId) -> Self::Output {
        rhs.0.x += self.x() * CHUNK_SIZE;
        rhs.0.y += self.y() * CHUNK_SIZE;
        rhs.0.z += self.z() * CHUNK_SIZE;
        rhs
    }
}

impl From<ChunkId> for bevy::asset::HandleId {
    fn from(value: ChunkId) -> Self {
        use std::hash::Hash;
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::default();
        value.hash(&mut hasher);
        bevy::asset::HandleId::Id(
            uuid::uuid!("0c7f7a1b-f6ca-4006-8032-d3a0bbdcb659"),
            hasher.finish(),
        )
    }
}

impl std::ops::Sub<ChunkId> for BlockId {
    type Output = BlockId;
    fn sub(mut self, rhs: ChunkId) -> Self::Output {
        self.0.x -= rhs.x() * CHUNK_SIZE;
        self.0.y -= rhs.y() * CHUNK_SIZE;
        self.0.z -= rhs.z() * CHUNK_SIZE;
        self
    }
}

impl Debug for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Chunk({}, {}, {})",
            self.x(),
            self.y(),
            self.z()
        ))
    }
}

impl PartialEq<i32> for ChunkId {
    fn eq(&self, other: &i32) -> bool {
        self.x() == *other && self.y() == *other && self.z() == *other
    }
}

impl PartialOrd<i32> for ChunkId {
    fn lt(&self, other: &i32) -> bool {
        self.x() < *other || self.y() < *other || self.z() < *other
    }
    fn gt(&self, other: &i32) -> bool {
        self.x() > *other || self.y() > *other || self.z() > *other
    }
    fn partial_cmp(&self, _: &i32) -> Option<std::cmp::Ordering> {
        None
    }
}

#[derive(Component, Clone, Copy)]
pub struct BlockId(IVec3);

impl BlockId {
    pub fn new(x: i32, y: i32, z: i32) -> BlockId {
        BlockId(IVec3::new(x, y, z))
    }

    pub fn x(&self) -> i32 {
        self.0.x
    }

    pub fn y(&self) -> i32 {
        self.0.y
    }

    pub fn z(&self) -> i32 {
        self.0.z
    }

    pub fn get(&self, direction: Direction) -> BlockId {
        match direction {
            Direction::Up => BlockId::new(self.x(), self.y() + 1, self.z()),
            Direction::Down => BlockId::new(self.x(), self.y() - 1, self.z()),
            Direction::Left => BlockId::new(self.x() - 1, self.y(), self.z()),
            Direction::Right => BlockId::new(self.x() + 1, self.y(), self.z()),
            Direction::Forward => BlockId::new(self.x(), self.y(), self.z() + 1),
            Direction::Back => BlockId::new(self.x(), self.y(), self.z() - 1),
        }
    }
}

impl From<BlockId> for ChunkId {
    fn from(value: BlockId) -> Self {
        let x = (value.0.x + (value.0.x < 0) as i32) / CHUNK_SIZE - (value.0.x < 0) as i32;
        let y = (value.0.y + (value.0.y < 0) as i32) / CHUNK_SIZE - (value.0.y < 0) as i32;
        let z = (value.0.z + (value.0.z < 0) as i32) / CHUNK_SIZE - (value.0.z < 0) as i32;
        ChunkId(IVec3::new(x,y,z))
    }
}

impl PartialEq<i32> for BlockId {
    fn eq(&self, other: &i32) -> bool {
        self.x() == *other && self.y() == *other && self.z() == *other
    }
}

impl PartialOrd<i32> for BlockId {
    fn lt(&self, other: &i32) -> bool {
        self.x() < *other || self.y() < *other || self.z() < *other
    }
    fn gt(&self, other: &i32) -> bool {
        self.x() > *other || self.y() > *other || self.z() > *other
    }
    fn partial_cmp(&self, _: &i32) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Debug for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Block({}, {}, {})",
            self.x(),
            self.y(),
            self.z()
        ))
    }
}

pub fn name_chunks(mut commands: Commands, chunks: Query<(Entity, &ChunkId), Changed<ChunkId>>) {
    for (chunk, id) in &chunks {
        commands
            .entity(chunk)
            .insert(Name::new(format!("{:?}", id)));
    }
}

#[test]
fn test_block_to_chunk() {
    for x in -10..10 {
        for y in -10..10 {
            for z in -10..10 {
                let chunk_id = ChunkId::new(x, y, z);
                for bx in 0..CHUNK_SIZE {
                    for by in 0..CHUNK_SIZE {
                        for bz in 0..CHUNK_SIZE {
                            let block_id = BlockId::new(bx + x * CHUNK_SIZE, by + y * CHUNK_SIZE, bz + z * CHUNK_SIZE);
                            if ChunkId::from(block_id) != chunk_id {
                                panic!("{:?} | {:?} != {:?}", ChunkId::from(block_id), block_id, chunk_id);
                            }
                        }
                    }
                }
            }
        }
    }
}
