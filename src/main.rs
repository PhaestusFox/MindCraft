use bevy::prelude::*;
use bevy_editor_pls::prelude::*;
use bevy_rapier3d::prelude::*;
use blocks::BlockType;
use textures::TextureHandels;

mod textures;

mod blocks;

fn main() {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, EditorPlugin::default(), RapierPhysicsPlugin::<()>::default()));
    #[cfg(debug_assertions)]
    app.add_plugins(RapierDebugRenderPlugin::default());
    app.add_plugins(textures::TexturePlugin);
    app.add_systems(Startup, spawn_cube);
    app.run()
}

fn spawn_cube(
    mut commands: Commands,
    mut mesh: ResMut<Assets<Mesh>>,
    atlas: Res<TextureHandels>,
) {
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
    ].into_iter()));
    commands.spawn(PbrBundle {
        mesh: mesh.add(blocks::make_test_block_mesh(4, 3)),
        material: atlas.get(),
        ..Default::default()
    });
}
