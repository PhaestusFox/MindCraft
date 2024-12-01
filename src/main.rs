use bevy::{prelude::*, utils::HashMap};
// use bevy_editor_pls::prelude::*;
use avian3d::prelude::*;
use blocks::BlockType;
use prelude::ChunkId;
use settings::ViewDistance;
use textures::TextureHandles;

use crate::prelude::CHUNK_SIZE;

mod player_controller;

pub mod settings;

mod textures;
// mod world;

mod prelude;

mod blocks;

mod cam;

mod terrain;

mod components;

mod physics;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        PhysicsPlugins::default(),
        physics::PhysicsPlugin,
    ));
    #[cfg(debug_assertions)]
    app.add_systems(Update, frame_time);
    #[cfg(debug_assertions)]
    app.add_plugins((
        // PhysicsDebugPlugin::default(),
        bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        bevy::diagnostic::EntityCountDiagnosticsPlugin,
        // bevy_diagnostics_explorer::DiagnosticExplorerAgentPlugin,
    ));
    app.add_plugins((textures::TexturePlugin, cam::PlayerPlugin));
    app.add_systems(Startup, spawn_cube);
    app.add_systems(PreUpdate, components::name_chunks);
    // app.insert_resource(world::Map::new_with_seed(3))
    //     .init_resource::<ChunkMeshTasks>();
    app.add_plugins(terrain::TerrainPlugin);
    app.init_resource::<ViewDistance>();
    app.register_type::<ViewDistance>();
    // app.add_plugins(belly::prelude::BellyPlugin);
    app.insert_state(GameState::GenWorld);
    // app.add_plugins((menu::MenuPlugin));
    app.configure_sets(Update, Playing.run_if(not(in_state(GameState::MainMenu))));
    app.add_plugins(player_controller::PlayerPlugin);
    app.add_systems(Update, settings::change_view_distance);
    app.run();
}

fn spawn_cube(mut commands: Commands, mut mesh: ResMut<Assets<Mesh>>, atlas: Res<TextureHandles>) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(0., 256., 0.))
            .with_rotation(Quat::from_rotation_x(-0.3)),
    ));
    commands.queue(textures::MakeTextureAtlas::new(
        [
            BlockType::Bedrock,
            BlockType::Gravel,
            BlockType::Dirt,
            BlockType::Stone,
            BlockType::Sand,
            BlockType::GoldOre,
            BlockType::IronOre,
            BlockType::CoalOre,
            // BlockType::DeadBush,
            BlockType::Grass,
            BlockType::Water,
        ]
        .into_iter(),
    ));

    commands.spawn((
        Mesh3d(mesh.add(blocks::make_test_block_mesh(4, 3))),
        MeshMaterial3d(atlas.get_atlas()),
    ));
}

// #[derive(Resource, Default)]
// pub struct ChunkMeshTasks(HashMap<ChunkId, bevy::tasks::Task<Option<ChunkGenData>>>);

// impl ChunkMeshTasks {
//     pub fn cancel(&mut self, id: &ChunkId) {
//         self.0.remove(id);
//     }
// }

// fn gen_chunk_tasks(
//     chunks: Res<Map>,
//     atlas: Res<TextureHandles>,
//     mut tasks: ResMut<ChunkMeshTasks>,
// ) {
//     let pool = bevy::tasks::AsyncComputeTaskPool::get();
//     let new = chunks.take_new();
//     for id in new {
//         let id = id.clone();
//         let atlas = atlas.clone();
//         let map = chunks.clone();
//         tasks
//             .0
//             .insert(id, pool.spawn(Chunk::make_mesh(id, map, atlas)));
//     }
// }

// fn add_chunk_meshes(
//     mut commands: Commands,
//     mut tasks: ResMut<ChunkMeshTasks>,
//     mut assets: ResMut<Assets<Mesh>>,
//     map: Res<Map>,
//     textures: Res<TextureHandles>,
// ) {
//     let mut done = Vec::new();
//     for (id, task) in tasks.0.iter() {
//         if task.is_finished() {
//             done.push(*id);
//         }
//     }
//     for id in done {
//         let task = tasks.0.remove(&id).expect("I know its there");
//         let Some(data) = futures_lite::future::block_on(task.cancel()).expect("Is finished") else {
//             error!("Failed to gen chunk");
//             continue;
//         };
//         let e = map.get_entity(&id).expect("Chunk to be in world");
//         let Some(mut entity) = commands.get_entity(e) else {
//             trace!("chunk entity not it world");
//             continue;
//         };
//         entity.despawn_descendants();
//         if let Some(collider) = data.collider {
//             entity.with_children(|p| {
//                 p.spawn((
//                     if collider.shape().is::<avian3d::parry::shape::Cuboid>() {
//                         (
//                             Transform::from_translation(Vec3::splat(
//                                 (CHUNK_SIZE as f32 / 2.) - 0.5,
//                             )),
//                             Visibility::Visible,
//                         )
//                     } else {
//                         (Transform::IDENTITY, Visibility::Visible)
//                     },
//                     collider,
//                     Name::new("collider"),
//                 ));
//             });
//         }
//         if data.main_mesh.count_vertices() != 0 {
//             let _ = assets.insert(
//                 bevy::asset::AssetId::Uuid {
//                     uuid: uuid::Uuid::from_u128(id.to_u128()),
//                 },
//                 data.main_mesh,
//             );
//         }
//         if let Some(water) = data.water_mesh {
//             entity.with_children(|p| {
//                 p.spawn((
//                     PbrBundle {
//                         mesh: Mesh3d(assets.add(water)),
//                         material: MeshMaterial3d(textures.get_water()),
//                         ..Default::default()
//                     },
//                     Name::new("water"),
//                 ));
//             });
//         }
//     }
// }

fn frame_time(
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
    time: Res<Time>,
    mut last: Local<f32>,
) {
    if time.elapsed_secs() - *last < 1. {
        return;
    }
    *last = time.elapsed_secs();

    let Some(d) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME) else {
        return;
    };
    println!("Frame Time = {:?}", d.average());
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy, Hash, States)]
enum GameState {
    MainMenu,
    EscapeMenu,
    #[default]
    GenWorld,
    Playing,
}

//mod menu;

#[derive(Debug, SystemSet, Hash, PartialEq, Eq, Clone, Copy)]
struct Playing;
