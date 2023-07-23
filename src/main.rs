use bevy::{prelude::*, utils::{HashMap, HashSet}};
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::*;
use blocks::BlockType;
use prelude::ChunkId;
use textures::TextureHandles;
use world::chunk::Chunk;

mod textures;
mod world;

mod prelude;

mod blocks;

mod cam;

mod components;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.build().disable::<bevy::log::LogPlugin>(),
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
            mode: DebugRenderMode::COLLIDER_AABBS,
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
    app.add_systems(
        Update,
        (
            world::gen_view_chunks,
            gen_chunk_tasks.after(world::gen_view_chunks),
            world::hide_view_chunks,
            components::name_chunks,
            add_chunk_meshes,
        ),
    );
    app.insert_resource(world::Map::default())
    .init_resource::<ChunkMeshTasks>();
    app.add_systems(Update, add_colider);
    app.add_event::<MeshEvent>();
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
struct ChunkMeshTasks(HashMap<ChunkId, bevy::tasks::Task<Mesh>>);

fn gen_chunk_tasks(
    chunks: Query<(&ChunkId, &Chunk), Changed<Chunk>>,
    // mut meshs: ResMut<Assets<Mesh>>,
    atlas: Res<TextureHandles>,
    mut tasks: ResMut<ChunkMeshTasks>,
) {
    let pool = bevy::tasks::AsyncComputeTaskPool::get();
    for (id, data) in &chunks {
        let data = data.clone();
        let id = id.clone();
        let atlas = atlas.clone();
        tasks
            .0
            .insert(id, pool.spawn(async move { data.gen_mesh(&atlas) }));
    }
}

fn add_chunk_meshes(mut tasks: ResMut<ChunkMeshTasks>,
    mut assets: ResMut<Assets<Mesh>>,
    mut events: EventWriter<MeshEvent>,
) {
    let mut done = Vec::new();
    for (id, task) in tasks.0.iter() {
        if task.is_finished() {
            done.push(*id);
        }
    }
    for id in done {
        let task = tasks.0.remove(&id).expect("I know its there");
        events.send(MeshEvent::GenCollider(id));
        let _ = assets.set(
            id,
            futures_lite::future::block_on(task.cancel()).expect("Is finished"),
        );
    }
}

#[derive(Debug, Event)]
enum MeshEvent {
    GenCollider(ChunkId),
}

fn add_colider(
    mut chunks: Query<(&ChunkId, &mut Collider, &Chunk)>,
    mut events: EventReader<MeshEvent>,
    mut local: Local<HashSet<ChunkId>>,
    assets: Res<Assets<Mesh>>,
) {
    for e in events.iter() {
        match e {
            MeshEvent::GenCollider(id) => {local.insert(*id);},
        }
    }
    for (id, mut collider, chunk) in &mut chunks {
        if !local.contains(id) {continue;}
        *collider = chunk.gen_collider();
        local.remove(id);
    }
}

fn frame_time(
    diagnostics: Res<bevy::diagnostic::DiagnosticsStore>,
) {
    let Some(d) = diagnostics.get(bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME) else {return;};
    println!("Frame Time = {:?}", d.average());
}