use bevy::{math::vec2, prelude::*};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(sprite_animation_system.system())
        .run();
}

fn setup(
    mut cmds: Commands,
    asset: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let tex: Handle<Texture> = asset.load("image.png");
    let atlas = TextureAtlas::from_grid_with_padding(tex.clone(), vec2(48.0, 48.0), 8, 8, vec2(2.0, 2.0));
    let atlas_handle = atlases.add(atlas);

    let material_handle = materials.add(ColorMaterial {
        texture: Some(tex),
        ..Default::default()
    });

    let mut cam = OrthographicCameraBundle::new_2d();
    cam.orthographic_projection.scale = 0.25;
    cmds.spawn_bundle(cam);

    cmds.spawn()
        .insert(Animation {
            row: 0,
            col: 0,
            flip_x: false,
            flip_y: false,
        })
        .insert_bundle(SpriteSheetBundle {
            texture_atlas: atlas_handle,
            ..Default::default()
        });

    cmds.spawn().insert_bundle(SpriteBundle {
        material: material_handle,
        ..Default::default()
    });
}

#[derive(Clone, Default)]
struct Animation {
    row: u32,
    col: u32,
    flip_x: bool,
    flip_y: bool,
}

fn sprite_animation_system(
    mut belts: Query<(&mut Animation, &mut TextureAtlasSprite)>,
    time: Res<Time>,
) {
    let time = time.seconds_since_startup();
    let anim_col = (time.fract() * 8.0) as u32;

    for (mut anim, mut sprite) in belts.iter_mut() {
        if sprite.flip_x != anim.flip_x {
            sprite.flip_x = anim.flip_x;
        }
        if sprite.flip_y != anim.flip_y {
            sprite.flip_y = anim.flip_y;
        }
        if anim.col != anim_col {
            anim.col = anim_col;
        }

        let new_index = anim.col + anim.row * 8;
        if sprite.index != new_index {
            sprite.index = new_index;
        }
    }
}
