use bevy::{prelude::*, utils::HashMap};
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::*;
use blocks::BlockType;
use prelude::ChunkId;
use settings::ViewDistance;
use textures::TextureHandles;
use world::{chunk::{Chunk, ChunkGenData}, Map};

pub mod settings;

mod textures;
mod world;

mod prelude;

mod blocks;

mod cam;

mod components;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.build()
        // .disable::<bevy::log::LogPlugin>()
        .set(AssetPlugin{
            watch_for_changes: bevy::asset::ChangeWatcher::with_delay(std::time::Duration::from_millis(50)),
            ..Default::default()
        }),
        EditorPlugin::default(),
        RapierPhysicsPlugin::<()>::default(),
    ));
    #[cfg(debug_assertions)]
    app.add_systems(FixedUpdate, frame_time);
    app.insert_resource(FixedTime::new_from_secs(1.));
    #[cfg(debug_assertions)]
    app.add_plugins((
        // it is fast unless debug rendering it on, it is too many lines so im only going to show the
        // AABB not the acctual shape for now
        RapierDebugRenderPlugin {
            mode: DebugRenderMode::all(),
            ..Default::default()
        },
        bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        bevy::diagnostic::EntityCountDiagnosticsPlugin,
        // bevy_diagnostics_explorer::DiagnosticExplorerAgentPlugin,
    ));
    app.add_plugins((textures::TexturePlugin, cam::PlayerPlugin));
    app.add_systems(Startup, spawn_cube);
    app.insert_resource(world::WorldDescriptior::new(3));
    app.add_systems(Startup, world::gen_start_chunks);
    app.add_systems(PreUpdate, components::name_chunks);
    app.add_systems(
        Update,
        (
            world::gen_view_chunks,
            gen_chunk_tasks.after(world::gen_view_chunks),
            world::hide_view_chunks,
            add_chunk_meshes,
        ),
    );
    app.insert_resource(world::Map::default())
    .init_resource::<ChunkMeshTasks>();
    app.init_resource::<ViewDistance>();
    app.register_type::<ViewDistance>();
    app.run()
}

fn spawn_cube(mut commands: Commands, mut mesh: ResMut<Assets<Mesh>>, atlas: Res<TextureHandles>) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(0., 256., 0.))
            .with_rotation(Quat::from_rotation_x(-0.3)),
        directional_light: DirectionalLight {
            // shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });

    commands.add(textures::MakeTextureAtlas::new(
        [
            BlockType::Bedrock,
            BlockType::Gravel,
            BlockType::Dirt,
            BlockType::Stone,
            BlockType::Sand,
            BlockType::GoldOre,
            BlockType::IronOre,
            BlockType::CoalOre,
            BlockType::DeadBush,
            BlockType::Grass,
            BlockType::Water,
        ]
        .into_iter(),
    ));

    commands.spawn(PbrBundle {
        mesh: mesh.add(blocks::make_test_block_mesh(4, 3)),
        material: atlas.get_atlas(),
        ..Default::default()
    });
}

#[derive(Resource, Default)]
pub struct ChunkMeshTasks(HashMap<ChunkId, bevy::tasks::Task<Option<ChunkGenData>>>);

impl ChunkMeshTasks {
    pub fn cancel(&mut self, id: &ChunkId) {
        self.0.remove(id);
    }
}

fn gen_chunk_tasks(
    chunks: Res<Map>,
    atlas: Res<TextureHandles>,
    mut tasks: ResMut<ChunkMeshTasks>,
) {
    let pool = bevy::tasks::AsyncComputeTaskPool::get();
    let new = chunks.take_new();
    for id in new {
        let id = id.clone();
        let atlas = atlas.clone();
        let map = chunks.clone();
        tasks
            .0
            .insert(id, pool.spawn(Chunk::make_mesh(id, map, atlas)));
    }
}

fn add_chunk_meshes(
    mut commands: Commands,
    mut tasks: ResMut<ChunkMeshTasks>,
    mut assets: ResMut<Assets<Mesh>>,
    map: Res<Map>,
    textures: Res<TextureHandles>,
) {
    let mut done = Vec::new();
    for (id, task) in tasks.0.iter() {
        if task.is_finished() {
            done.push(*id);
        }
    }
    for id in done {
        let task = tasks.0.remove(&id).expect("I know its there");
        let Some(data) = futures_lite::future::block_on(task.cancel()).expect("Is finished") else {
            error!("Faild to gen chunk"); continue;};
        let e = map.get_entity(&id).expect("Chunk to be in world");
        if let Some(collider) = data.collider {
            commands.entity(e).insert(collider);
        }
        if let Some(mesh) = data.main_mesh {
            let _ = assets.set(id, mesh);
        }
        if let Some(water) = data.water_mesh {
            commands.entity(e).with_children(|p| {
                p.spawn(PbrBundle {
                    mesh: assets.add(water),
                    material: textures.get_water(),
                    ..Default::default()
                });
            });
        }
    }
}

fn frame_time(
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
) {
    let Some(d) = diagnostics.get(bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME) else {return;};
    println!("Frame Time = {:?}", d.average());
}