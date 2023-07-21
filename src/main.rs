use bevy::{prelude::*, utils::HashMap};
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::*;
use blocks::BlockType;
use prelude::ChunkId;
use textures::TextureHandels;
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
        DefaultPlugins,
        EditorPlugin::default(),
        RapierPhysicsPlugin::<()>::default(),
    ));
    #[cfg(debug_assertions)]
    app.add_plugins(RapierDebugRenderPlugin::default());
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
        ),
    );
    app.insert_resource(world::Map::default());
    app.run()
}

fn spawn_cube(mut commands: Commands, mut mesh: ResMut<Assets<Mesh>>, atlas: Res<TextureHandels>) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(0., 256., 0.))
            .with_rotation(Quat::from_rotation_x(-0.3)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
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

#[derive(Resource)]
struct ChunkMeshTasks(bevy::tasks::Task<(ChunkId, Mesh)>);

fn gen_chunk_tasks(
    mut chunks: Query<(&ChunkId, &Chunk, &mut Handle<Mesh>), Changed<Chunk>>,
    mut meshs: ResMut<Assets<Mesh>>,
    atlas: Res<TextureHandels>,
) {
    let num = std::sync::Mutex::new(0);
    let start = std::time::Instant::now();
    let map = std::sync::Mutex::new(HashMap::new());
    chunks.par_iter().for_each(|(id, chunk, _)| {
        *num.lock().unwrap() += 1;
        let mesh = chunk.gen_mesh(&atlas);
        map.lock().unwrap().insert(*id, mesh);
    });
    let mut map = map.lock().unwrap();
    for (id, _, mut handle) in &mut chunks {
        let Some(mesh) = map.remove(id) else {error!("Missing Mesh?"); continue;};
        *handle = meshs.add(mesh);
    }
    let num = *num.lock().unwrap();
    if num > 0 {
        println!(
            "generated {} chunks in {}; avr = {}",
            num,
            start.elapsed().as_secs_f64(),
            start.elapsed().as_secs_f64() / num as f64
        );
    }
}
