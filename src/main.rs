use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::*;
use blocks::BlockType;
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
    app.add_plugins((DefaultPlugins, EditorPlugin::default(), RapierPhysicsPlugin::<()>::default()));
    #[cfg(debug_assertions)]
    app.add_plugins(RapierDebugRenderPlugin::default());
    app.add_plugins((textures::TexturePlugin, cam::PlayerPlugin));
    app.add_systems(Startup, spawn_cube);
    app.add_systems(Update, gen_chunk_mesh);
    app.insert_resource(world::WorldDescriptior::new(3));
    app.add_systems(Startup, world::gen_start_chunks);
    app.add_systems(Update, (world::gen_view_chunks, world::hide_view_chunks));
    app.insert_resource(world::Map::default());
    app.run()
}

fn spawn_cube(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    atlas: Res<TextureHandels>,
) {
    commands.spawn(
        DirectionalLightBundle {
            transform: Transform::from_translation(Vec3::new(0., 256., 0.)).with_rotation(Quat::from_rotation_x(-0.3)),
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        }
    );

    commands.add(textures::MakeTextureAtlas::new([
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
    ].into_iter()));

    commands.spawn(PbrBundle {
        mesh: mesh.add(blocks::make_test_block_mesh(4, 3)),
        material: atlas.get_atlas(),
        ..Default::default()
    });
}

fn gen_chunk_mesh(
    mut chunks: Query<(&Chunk, &mut Handle<Mesh>), Changed<Chunk>>,
    mut meshs: ResMut<Assets<Mesh>>,
    atlas: Res<TextureHandels>,
) {
    for (chunk, mut handle) in &mut chunks {
        *handle = meshs.add(chunk.gen_mesh(&atlas));
    }
}
