use bevy::{prelude::*, utils::HashSet};
use crate::{prelude::*, cam::FlyCam};

use self::chunk::{Position, Chunk};

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
        WorldDescriptior {
            seed,
            rng,
        }
    }
}

#[derive(Debug, Resource, Default)]
pub struct Map(HashSet<Position>);

pub fn gen_start_chunks(
    mut commands: Commands,
    world_descriptior: Res<WorldDescriptior>,
    matt: Res<TextureHandels>,
) {
    for y in 0..5 {
        for z in 0..5 {
            for x in 0..5 {
                commands.spawn((PbrBundle {
                    transform: Transform::from_translation(Vec3::new((x * CHUNK_SIZE) as f32, (y * CHUNK_SIZE) as f32, (z * CHUNK_SIZE) as f32)),
                    material: matt.get(),
                    ..Default::default()
                }, chunk::Chunk::new(chunk::Position::new(x, y, z), &world_descriptior.rng, world_descriptior.seed)));
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
) {
    let player = player.single().translation;
    let center = Position::new((player.x / CHUNK_SIZE as f32) as isize, 0, (player.z / CHUNK_SIZE as f32) as isize);
    for z in -5..5 {
        for x in -5..5 {
            let pos = Position::new(center.x + x, 0, center.z + z);
            if map.0.contains(&pos) {continue;}
            map.0.insert(pos);
            for y in 0..5 {
                let pos = Position::new(center.x + x, y, center.z + z);
                commands.spawn((PbrBundle {
                    transform: Transform::from_translation(Vec3::new((pos.x * CHUNK_SIZE) as f32, (pos.y * CHUNK_SIZE) as f32, (pos.z * CHUNK_SIZE) as f32)),
                    material: matt.get(),
                    ..Default::default()
                }, chunk::Chunk::new(pos, &world_descriptior.rng, world_descriptior.seed)));
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