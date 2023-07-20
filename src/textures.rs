use bevy::{prelude::*, ecs::system::Command};

use crate::blocks::BlockType;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, build_texture_atlas.run_if(resource_exists::<TextureAtlasBuilder>()));
        app.init_resource::<TextureHandels>();
    }
}

#[derive(Resource)]
struct TextureAtlasBuilder(Vec<Handle<Image>>);

pub struct MakeTextureAtlas(Vec<BlockType>);
impl MakeTextureAtlas {
    pub fn new(blocks: impl Iterator<Item = BlockType>) -> Self {
        MakeTextureAtlas(blocks.collect())
    }
}
impl Command for MakeTextureAtlas {
    fn apply(self, world: &mut World) {
        let mut need_textures = Vec::with_capacity(self.0.len());
        let asset_server = world.resource::<AssetServer>();
        for block in self.0 {
            for path in block.get_texture_paths() {
                need_textures.push(asset_server.load::<_, &str>(path));
            }
        }
        world.insert_resource(TextureAtlasBuilder(need_textures));
    }
}

#[derive(Resource)]
pub struct TextureHandels(Handle<StandardMaterial>);

impl TextureHandels {
    pub fn get(&self) -> Handle<StandardMaterial> {
        self.0.clone()
    }
}

impl FromWorld for TextureHandels {
    fn from_world(world: &mut World) -> Self {
        let main_image = world.resource_mut::<Assets<Image>>().get_handle("MainAtlas");
        let texture = world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial {
            base_color_texture: Some(main_image),
            metallic: 0.,
            reflectance: 0.,
            alpha_mode: AlphaMode::Mask(0.1),
            ..Default::default()
        });
        TextureHandels(texture)
    }
}

fn build_texture_atlas(
    mut commands: Commands,
    atlas_builder: Res<TextureAtlasBuilder>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    if let bevy::asset::LoadState::Loaded = asset_server.get_group_load_state(atlas_builder.0.iter().map(|h| h.id())) {
        
    } else {
        return;
    }
    let first = images.get(&atlas_builder.0[0]).expect("All images to be built");
    let atlas_size = (atlas_builder.0.len() as f32).sqrt().ceil() as usize;
    let width = first.texture_descriptor.size.width as usize;
    let format = first.texture_descriptor.format;
    let pixed_size = format.block_size(None).unwrap() as usize;
    let mut atlas_data = vec![0; (atlas_size * atlas_size * width * width * pixed_size) as usize];
    'y: for y in 0..atlas_size {
        for x in 0..atlas_size {
            let Some(handle) = atlas_builder.0.get(y * atlas_size + x) else {break 'y;};
            let Some(image) = images.get(handle) else {continue;};
            fill_from(&mut atlas_data, x, y, atlas_size, width, &image.data, pixed_size);
        }
    }
    let size = bevy::render::render_resource::Extent3d {
        width: (width * atlas_size) as u32,
        height: (width * atlas_size) as u32,
        depth_or_array_layers: 1
    };
    let _ = images.set("MainAtlas", Image::new(size, bevy::render::render_resource::TextureDimension::D2, atlas_data, format));
    commands.remove_resource::<TextureAtlasBuilder>();
}

fn fill_from(data: &mut Vec<u8>, offset_x: usize, offset_y: usize, atlas_size: usize, block_size: usize, image: &[u8], pixel_size: usize) {
    let x_off = offset_x * block_size;
    let y_off = offset_y * block_size * block_size * atlas_size;
    for y in 0..block_size {
        for x in 0..block_size {
            let index = ((y_off + y * block_size * atlas_size) + (x_off + x)) * pixel_size;
            let imdex = ((y * block_size) + x) * pixel_size;
            data[index] = image[imdex];
            data[index + 1] = image[imdex + 1];
            data[index + 2] = image[imdex + 2];
            data[index + 3] = image[imdex + 3];
        }
    }
}