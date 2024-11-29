use std::sync::{Arc, RwLock, RwLockReadGuard};

use bevy::{
    prelude::*,
    tasks::Task,
    utils::{HashMap, HashSet},
};
use noise::NoiseFn;
use rand::{Rng, SeedableRng};

use crate::{player_controller::Player, prelude::*, settings::ViewDistance};

use bevy::prelude::*;

mod chunk;

use chunk::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Seed(3))
            .add_systems(Update, (que_chunks, spawn_visable_chunks))
            .add_systems(PreUpdate, (start_gen_chunks, start_mesh_chunks))
            .add_systems(PostUpdate, (finish_gen_chunks, finish_mesh_chunks))
            .add_systems(First, update_can_mesh)
            .init_resource::<MapDescriptor>()
            .init_resource::<Map>();
    }
}

#[derive(Resource)]
struct MapDescriptor {
    noise: noise::Fbm<noise::OpenSimplex>,
    rng: rand::rngs::StdRng,
}

impl FromWorld for MapDescriptor {
    fn from_world(world: &mut World) -> Self {
        let seed = world.get_resource::<Seed>().unwrap_or(&Seed(3)).0;
        let mut noise = noise::Fbm::<noise::OpenSimplex>::new(((seed >> 32) ^ seed) as u32);
        let rng = rand::rngs::StdRng::seed_from_u64(seed);
        noise.frequency = 0.2;
        noise.persistence = 0.50;
        noise.octaves = 4;
        MapDescriptor { noise, rng }
    }
}

#[derive(Resource, Deref)]
pub struct Seed(u64);

#[derive(Resource, Default)]
pub struct Map {
    pub generate_tasks: HashMap<ChunkId, Task<Result<Chunk, GenError>>>,
    pub mesh_task: HashMap<Entity, Task<Result<Mesh, MeshError>>>,
    pub chunk_data: Arc<RwLock<ChunkData>>,
    pub has_data: HashSet<ChunkId>,
    pub can_mesh: HashSet<ChunkId>,
    pub to_gen: HashSet<ChunkId>,
    pub to_mesh: HashMap<ChunkId, Entity>,
    pub id_to_entity: HashMap<ChunkId, Entity>,
}

impl Map {
    fn que_chunk(&mut self, entity: Entity, id: ChunkId) {
        self.add_neighbors(id);
        if !self.to_gen.contains(&id) {
            self.to_gen.insert(id);
        }
        self.to_mesh.insert(id, entity);
    }

    fn add_neighbors(&mut self, id: ChunkId) {
        for n in id.neighbors() {
            // chunk bellow bedrock
            if n.y < 0 {
                continue;
            }
            // chunk above build limit
            if n.y > 4 {
                continue;
            }
            // chunk already in que
            if self.to_gen.contains(&n) {
                continue;
            }
            // chunk already in generated
            if self.has_data.contains(&n) {
                continue;
            }
            self.to_gen.insert(n);
        }
    }

    fn update_can_mesh(&mut self) {
        self.can_mesh.clear();
        'main: for id in self.to_mesh.keys() {
            if !self.is_usable(id) {
                continue;
            }
            for n in id.neighbors() {
                if !self.is_usable(&n) {
                    continue 'main;
                }
            }
            self.can_mesh.insert(*id);
        }
    }

    fn is_usable(&self, id: &ChunkId) -> bool {
        id.y < 0 || id.y > 4 || self.has_data.contains(id)
    }

    fn get_entity(&self, id: &ChunkId) -> Option<Entity> {
        self.id_to_entity.get(id).cloned()
    }

    fn add_chunk(&mut self, id: ChunkId, chunk: Chunk) {
        if id.y > 4 || id.y < 0 {
            panic!()
        }
        self.chunk_data.write().unwrap().set(id, chunk);
        self.has_data.insert(id);
    }

    fn remove_chunk(&mut self, id: &ChunkId) {
        self.chunk_data.write().unwrap().remove(id);
        self.has_data.remove(id);
        self.id_to_entity.remove(id);
    }

    pub fn get_block(&self, mut block: BlockId) -> BlockType {
        let chunk: ChunkId = block.into();
        let block = block.as_local();
        self.chunk_data
            .read()
            .unwrap()
            .get(&chunk)
            .get_block(block.x, block.y, block.z)
    }
}

#[derive(Default)]
struct ChunkData(HashMap<ChunkId, Chunk>);

impl ChunkData {
    fn get(&self, id: &ChunkId) -> &Chunk {
        if id.y > 4 || id.y < 0 {
            &Chunk::EMPTY
        } else {
            self.0.get(id).unwrap_or(&Chunk::EMPTY)
        }
    }
    fn try_get(&self, id: &ChunkId) -> Option<&Chunk> {
        self.0.get(id)
    }
    fn set(&mut self, id: ChunkId, data: Chunk) {
        if id.y < 0 || id.y > 4 {
            panic!()
        }
        self.0.insert(id, data);
    }
    fn remove(&mut self, id: &ChunkId) {
        self.0.remove(id);
    }
}

fn que_chunks(mut map: ResMut<Map>, added: Query<(Entity, &ChunkId), Added<ChunkId>>) {
    for (entity, id) in &added {
        map.que_chunk(entity, *id);
    }
}

fn start_gen_chunks(mut map: ResMut<Map>, world: Res<MapDescriptor>) {
    if map.to_gen.is_empty() {
        return;
    }
    let pool = bevy::tasks::AsyncComputeTaskPool::get();
    let mut tasks = std::mem::take(&mut map.generate_tasks);
    for id in map.to_gen.drain() {
        tasks.insert(
            id,
            pool.spawn(Chunk::new(world.noise.clone(), id.0, world.rng.clone())),
        );
    }
    std::mem::replace(&mut map.generate_tasks, tasks);
}

fn finish_gen_chunks(mut map: ResMut<Map>) {
    let mut finished = map
        .generate_tasks
        .iter()
        .filter(|(&k, &ref v)| v.is_finished())
        .map(|(k, _)| *k)
        .collect::<Vec<_>>();
    for id in finished {
        let task = map.generate_tasks.remove(&id).unwrap();
        let chunk = match futures_lite::future::block_on(task.cancel()).expect("Is finished") {
            Ok(chunk) => chunk,
            Err(e) => {
                error!("{}", e);
                continue;
            }
        };
        map.add_chunk(id, chunk);
    }
}

fn start_mesh_chunks(mut map: ResMut<Map>, text_atlas: Res<TextureHandles>) {
    if map.to_mesh.is_empty() {
        return;
    }
    let pool = bevy::tasks::AsyncComputeTaskPool::get();
    let ready = map
        .to_mesh
        .iter()
        .filter(|(&k, &v)| map.can_mesh.contains(&k))
        .map(|(k, v)| (*k, *v))
        .collect::<Vec<_>>();
    let mut tasks = std::mem::take(&mut map.mesh_task);
    for (id, target) in ready {
        println!("starting meshing of {:?}", id);
        map.id_to_entity.insert(id, target);
        map.to_mesh.remove(&id);
        tasks.insert(
            target,
            pool.spawn(Chunk::gen_mesh(
                ChunkId(*id),
                map.chunk_data.clone(),
                text_atlas.clone(),
            )),
        );
    }
    std::mem::replace(&mut map.mesh_task, tasks);
}

fn finish_mesh_chunks(
    mut commands: Commands,
    mut map: ResMut<Map>,
    asset_server: Res<AssetServer>,
) {
    let mut finished = map
        .mesh_task
        .iter()
        .filter(|(&k, &ref v)| v.is_finished())
        .map(|(k, _)| *k)
        .collect::<Vec<_>>();
    for id in finished {
        let task = map.mesh_task.remove(&id).unwrap();
        info!("mesh for {} done", id);
        let mesh = match futures_lite::future::block_on(task.cancel()).expect("Is finished") {
            Ok(chunk) => chunk,
            Err(e) => {
                error!("{}", e);
                continue;
            }
        };
        commands.entity(id).insert(Mesh3d(asset_server.add(mesh)));
    }
}

fn update_can_mesh(mut map: ResMut<Map>) {
    map.update_can_mesh();
}

fn spawn_visable_chunks(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    map: Res<Map>,
    matt: Res<TextureHandles>,
    view_distance: Res<ViewDistance>,
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
            if map.get_entity(&pos).is_some() || map.to_mesh.contains_key(&pos) {
                continue;
            }
            println!("add to spawn chunk");
            for y in 0..5 {
                let pos = ChunkId::new(center.x() + x, y, center.z() + z);
                let entity = commands
                    .spawn((
                        PbrBundle {
                            transform: Transform::from_translation(Vec3::new(
                                (pos.x() * CHUNK_SIZE) as f32,
                                (pos.y() * CHUNK_SIZE) as f32,
                                (pos.z() * CHUNK_SIZE) as f32,
                            )),
                            material: MeshMaterial3d(matt.get_atlas()),
                            mesh: Mesh3d(Handle::weak_from_u128(pos.to_u128())),
                            ..Default::default()
                        },
                        pos,
                    ))
                    .id();
            }
        }
    }
}
