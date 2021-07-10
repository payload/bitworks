use bevy::{math::vec2, prelude::*};

////////////

pub struct BeltAtlasHandle(Handle<TextureAtlas>);
impl std::ops::Deref for BeltAtlasHandle {
    type Target = Handle<TextureAtlas>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn load_belt_atlas(
    mut cmds: Commands,
    asset: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let tex: Handle<Texture> = asset.load("belt_atlas.png");
    let atlas = TextureAtlas::from_grid_with_padding(tex, vec2(48.0, 48.0), 8, 8, vec2(2.0, 2.0));
    cmds.insert_resource(BeltAtlasHandle(atlases.add(atlas)));
}

////////////

pub struct ItemAtlasHandle(Handle<TextureAtlas>);
impl std::ops::Deref for ItemAtlasHandle {
    type Target = Handle<TextureAtlas>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn load_item_texture(
    mut cmds: Commands,
    asset: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let tex: Handle<Texture> = asset.load("item.png");
    let atlas = TextureAtlas::from_grid_with_padding(tex, vec2(48.0, 48.0), 1, 1, Vec2::ZERO);
    cmds.insert_resource(ItemAtlasHandle(atlases.add(atlas)));
}
