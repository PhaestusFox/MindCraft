use crate::{cam::FlyCam, prelude::*};
use bevy::{prelude::*, utils::HashSet};
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

#[derive(Debug, Resource, Default)]
pub struct Map(HashSet<ChunkId>);

pub fn gen_start_chunks(
    mut commands: Commands,
    world_descriptior: Res<WorldDescriptior>,
    mut map: ResMut<Map>,
    matt: Res<TextureHandels>,
    asset_server: Res<AssetServer>,
) {
    for y in 0..5 {
        for z in -5..5 {
            for x in -5..5 {
                let id = ChunkId::new(x, y, z);
                map.0.insert(id);
                commands.spawn((
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
                    chunk::Chunk::new(id, &world_descriptior.rng, world_descriptior.seed),
                    id,
                    RigidBody::Fixed,
                    Collider::cuboid((CHUNK_SIZE / 2) as f32, (CHUNK_SIZE / 2) as f32, (CHUNK_SIZE / 2) as f32),
                ));
            }
        }
    }
}

pub fn gen_view_chunks(
    mut commands: Commands,
    mut map: ResMut<Map>,
    player: Query<&Transform, With<FlyCam>>,
    world_descriptior: Res<WorldDescriptior>,
    matt: Res<TextureHandels>,
    asset_server: Res<AssetServer>,
) {
    return;
    let player = player.single().translation;
    let center = ChunkId::new(
        (player.x / CHUNK_SIZE as f32) as i32,
        0,
        (player.z / CHUNK_SIZE as f32) as i32,
    );
    for z in -5..5 {
        for x in -5..5 {
            let pos = ChunkId::new(center.x() + x, 0, center.z() + z);
            if map.0.contains(&pos) {
                continue;
            }
            map.0.insert(pos);
            for y in 0..5 {
                let pos = ChunkId::new(center.x() + x, y, center.z() + z);
                commands.spawn((
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
                    chunk::Chunk::new(pos, &world_descriptior.rng, world_descriptior.seed),
                    pos,
                    RigidBody::Fixed,
                    Collider::cuboid((CHUNK_SIZE / 2) as f32, (CHUNK_SIZE / 2) as f32, (CHUNK_SIZE / 2) as f32),
                ));
            }
        }
    }
}

pub fn hide_view_chunks(
    mut chunks: Query<(&mut Visibility, &Transform), With<Chunk>>,
    player: Query<&Transform, With<FlyCam>>,
) {
    let player = player.single().translation;
    for (mut vis, pos) in &mut chunks {
        *vis = if pos.translation.distance(player) > CHUNK_SIZE as f32 * VIEW_DISTANCE {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}
