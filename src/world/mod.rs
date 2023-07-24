use crate::{cam::FlyCam, prelude::*, textures::TextureHandles};
use bevy::{prelude::*, utils::{HashMap, HashSet}};
use bevy_rapier3d::prelude::*;
use indexmap::IndexMap;
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

#[derive(Resource, Clone)]
pub struct Map(std::sync::Arc<std::sync::RwLock<MapInternal>>, std::sync::Arc<std::thread::JoinHandle<()>>);

struct MapInternal {
    descriptior: WorldDescriptior,
    chunks: HashMap<ChunkId, Chunk>,
    can_mesh: HashSet<ChunkId>,
    to_gen: IndexMap<ChunkId, Option<Entity>>,
}

impl MapInternal {
    pub fn get_block(&self, block: BlockId) -> BlockType {
        let chunk = ChunkId::from(block);
        let block = block - chunk;
        let Some(chunk) = self.get_chunk(&chunk) else {
            return BlockType::Air;
        };
        chunk.get_block(block)
    }

    pub fn get_chunk(&self, id: &ChunkId) -> Option<&Chunk> {
        self.chunks.get(id)
    }

    pub fn get_entity(&self, id: &ChunkId) -> Option<Entity> {
        self.chunks.get(id).and_then(|e| e.entity)
    }

    fn gen_chunk(&self, id: ChunkId) -> Option<Chunk> {
        if self.chunks.contains_key(&id) {return None;}
        println!("gen {:?}", id);
        Some(chunk::Chunk::new(id, &self.descriptior.rng, self.descriptior.seed))
    }

    fn get_or_gen_chunk_mut(&mut self, id: ChunkId) -> &mut Chunk {
        if !self.chunks.contains_key(&id) {
            let chunk = self.gen_chunk(id).expect("Chunk Not in map");
            self.chunks.insert(id, chunk);
        }
        self.chunks.get_mut(&id).expect("Add missing chunk")
    }

    pub fn gen_or_update_chunk(&mut self, id: ChunkId, entity: Entity) {
        for n in id.neighbours() {
            if n.y() > 4 || n.y() < 0 {continue;}
            self.add_to_gen(n, None);
        }

        self.to_gen.insert(id, Some(entity));
    }

    fn add_to_gen(&mut self, id: ChunkId, entity: Option<Entity>) {
        if self.chunks.contains_key(&id) {
            return;
        }
        if !self.to_gen.contains_key(&id) || entity.is_some() {
            self.to_gen.insert(id, entity);
        }
    }

    pub fn contains_chunk(&self, id: &ChunkId) -> bool {
        self.chunks.contains_key(id) || self.to_gen.contains_key(id)
    }

    pub fn remove_chunk(&mut self, id: &ChunkId) {
        self.chunks.remove(id);
    }

    pub fn take_new(&mut self) -> Vec<ChunkId> {
        self.can_mesh.drain().collect()
    }

    pub fn new_with_seed(seed: u64) -> MapInternal {
        MapInternal { descriptior: WorldDescriptior::new(seed), chunks: default(), can_mesh: default(), to_gen: default() }
    }
}

impl Map {
    pub fn to_gen(&self, id: &ChunkId) -> bool {
        self.0.read().unwrap().to_gen.contains_key(id)
    }

    pub fn get_entity(&self, id: &ChunkId) -> Option<Entity> {
        self.0.read().unwrap().get_entity(id)
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
    
    pub fn gen_or_update_chunk(&self, id: ChunkId, entity: Entity) {
        self.0.write().unwrap().gen_or_update_chunk(id, entity)
    }

    pub fn new() -> Map {
        use rand::Rng;
        Map::new_with_seed(rand::thread_rng().gen())
    }

    pub fn new_with_seed(seed: u64) -> Map {
        let internal = MapInternal::new_with_seed(seed);
        let internal = std::sync::Arc::new(std::sync::RwLock::new(internal));
        let rec_local = internal.clone();
        let join = std::thread::spawn(move || {
            loop {
            let mut made = Vec::with_capacity(10);
            let Ok(mut rec) = rec_local.try_write() else {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            };
            for _ in 0..10 {
                let (id, entity) = if let Some((id, entity)) = rec.to_gen.iter().next() {(*id, *entity)} else {break;};
                let chunk = rec.get_or_gen_chunk_mut(id);
                if let Some(entity) = entity {
                    chunk.set_entity(entity);
                    rec.can_mesh.insert(id);
                }
                made.push(id);
            }
            for made in made {
                rec.to_gen.remove(&made);
            }
            }
        });
        Map(internal, std::sync::Arc::new(join))
    }
}

pub fn gen_start_chunks(
    mut commands: Commands,
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
                map.gen_or_update_chunk(id, entity);
            }
        }
    }
}

pub fn gen_view_chunks(
    mut commands: Commands,
    map: Res<Map>,
    player: Query<&Transform, With<FlyCam>>,
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
            if map.get_entity(&pos).is_some() || map.to_gen(&pos) {
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
                map.gen_or_update_chunk(pos, entity);
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
