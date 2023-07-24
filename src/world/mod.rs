use crate::{cam::FlyCam, prelude::*, textures::TextureHandles};
use bevy::{prelude::*, utils::{HashMap, HashSet}};
use bevy_rapier3d::prelude::*;
use self::chunk::Chunk;

pub mod chunk;

#[derive(Resource)]
pub struct WorldDescriptior {
    seed: u64,
    rng: noise::Fbm<noise::SuperSimplex>,
}

impl WorldDescriptior {
    pub fn new(seed: u64) -> WorldDescriptior {
        let mut rng = noise::Fbm::new(((seed >> 32) ^ seed) as u32);
        rng.frequency = 0.05;
        rng.persistence = 0.25;
        WorldDescriptior { seed, rng }
    }
}

#[derive(Resource, Default, Clone)]
pub struct Map(std::sync::Arc<std::sync::RwLock<MapInternal>>);

#[derive(Default)]
struct MapInternal {
    chunks: HashMap<ChunkId, Chunk>,
    new: HashSet<ChunkId>,
}

impl MapInternal {
    pub fn get_chunk(&self, id: &ChunkId) -> Option<&Chunk> {
        self.chunks.get(id)
    }

    pub fn get_entity(&self, id: &ChunkId) -> Option<Entity> {
        self.chunks.get(id).and_then(|e| e.entity)
    }

    pub fn gen_chunk(&mut self, id: ChunkId, descriptior: &WorldDescriptior, entity: Entity) {
        let mut chunk = chunk::Chunk::new(id, &descriptior.rng, descriptior.seed);
        chunk.set_entity(entity);
        self.new.insert(id);
        self.chunks.insert(id, chunk);
    }

    pub fn contains_chunk(&self, id: &ChunkId) -> bool {
        self.chunks.contains_key(id)
    }

    pub fn remove_chunk(&mut self, id: &ChunkId) {
        self.chunks.remove(id);
    }

    pub fn take_new(&mut self) -> Vec<ChunkId> {
        self.new.drain().collect()
    }
}

impl Map {
    pub fn get_entity(&self, id: &ChunkId) -> Option<Entity> {
        self.0.read().unwrap().get_entity(id)
    }

    pub fn gen_chunk(&self, id: ChunkId, descriptior: &WorldDescriptior, entity: Entity) {
        self.0.write().unwrap().gen_chunk(id, descriptior, entity)
    }

    pub fn contains_chunk(&self, id: &ChunkId) -> bool {
        self.0.read().unwrap().contains_chunk(id)
    }

    pub fn remove_chunk(&self, id: &ChunkId) {
        self.0.write().unwrap().remove_chunk(id)
    }

    pub fn take_new(&self) -> Vec<ChunkId> {
        self.0.write().unwrap().take_new()
    }
}

pub fn gen_start_chunks(
    mut commands: Commands,
    world_descriptior: Res<WorldDescriptior>,
    map: Res<Map>,
    matt: Res<TextureHandles>,
    asset_server: Res<AssetServer>,
) {
    for y in 0..5 {
        for z in -5..5 {
            for x in -5..5 {
                let id = ChunkId::new(x, y, z);
                let entity = commands.spawn((
                    PbrBundle {
                        transform: Transform::from_translation(Vec3::new(
                            (x * CHUNK_SIZE) as f32,
                            (y * CHUNK_SIZE) as f32,
                            (z * CHUNK_SIZE) as f32,
                        )),
                        material: matt.get_atlas(),
                        mesh: asset_server.get_handle(id),
                        ..Default::default()
                    },
                    id,
                    RigidBody::Fixed,
                )).id();
                map.gen_chunk(id, &world_descriptior, entity);
            }
        }
    }
}

pub fn gen_view_chunks(
    mut commands: Commands,
    map: Res<Map>,
    player: Query<&Transform, With<FlyCam>>,
    world_descriptior: Res<WorldDescriptior>,
    matt: Res<TextureHandles>,
    asset_server: Res<AssetServer>,
    view_distance: Res<crate::settings::ViewDistance>,
) {
    let player = player.single().translation;
    let center = ChunkId::new(
        (player.x / CHUNK_SIZE as f32) as i32,
        0,
        (player.z / CHUNK_SIZE as f32) as i32,
    );
    let view_distance = view_distance.0;
    for z in -view_distance..view_distance {
        for x in -view_distance..view_distance {
            let pos = ChunkId::new(center.x() + x, 0, center.z() + z);
            if map.contains_chunk(&pos) {
                continue;
            }
            for y in 0..5 {
                let pos = ChunkId::new(center.x() + x, y, center.z() + z);
                let entity = commands.spawn((
                    PbrBundle {
                        transform: Transform::from_translation(Vec3::new(
                            (pos.x() * CHUNK_SIZE) as f32,
                            (pos.y() * CHUNK_SIZE) as f32,
                            (pos.z() * CHUNK_SIZE) as f32,
                        )),
                        material: matt.get_atlas(),
                        mesh: asset_server.get_handle(pos),
                        ..Default::default()
                    },
                    pos,
                    RigidBody::Fixed,
                )).id();
                map.gen_chunk(pos, &world_descriptior, entity);
            }
        }
    }
}

pub fn hide_view_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &ChunkId, &mut Visibility)>,
    player: Query<&Transform, With<FlyCam>>,
    view_distance: Res<crate::settings::ViewDistance>,
    map: Res<Map>,
    mut gen_tasks: ResMut<crate::ChunkMeshTasks>,
) {
    let player = ChunkId::from(player.single().translation);
    for (e, id, mut vis) in &mut chunks {
        let dis = id.flat_distance(player);
        if dis > view_distance.0 * 2 + 5 {
            gen_tasks.cancel(id);
            map.remove_chunk(id);
            commands.entity(e).despawn_recursive();
            continue;
        }
        *vis = if dis > 2 * view_distance.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
